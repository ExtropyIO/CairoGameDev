use crate::configs;
use crate::resources::*;
use bevy::{
    ecs::{event::Event, system::SystemState},
    log,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use hex;

use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use dojo_types::{primitive::Primitive, schema::Ty};
use dojo_world::contracts::WorldContractReader;

use futures_lite::future;
use std::thread;
use std::time::Duration;
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use url::Url;

use starknet::{
    accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount},
    core::{
        types::{BlockId, BlockTag, FieldElement},
        utils::{cairo_short_string_to_felt, get_selector_from_name},
    },
    providers::jsonrpc::HttpTransport,
    providers::JsonRpcClient,
    signers::{LocalWallet, SigningKey},
};

#[derive(Component)]
pub struct GameData {
    started: bool,
}

#[derive(Component)]
pub struct Game {
    pub game_id: Entity,
}

#[derive(Event)]
pub struct GameUpdate {
    pub game_update: Ty, // vector of field elements
}

#[derive(Event)]

struct GameStatus(Ty);

#[derive(Resource)]
pub struct DojoEnv {
    // block id to use for all contract calls
    block_id: BlockId,
    // address of the world contract
    world_address: FieldElement,
    // account to use for performing execution on the world contract
    account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
    provider: JsonRpcClient<HttpTransport>,
}

impl DojoEnv {
    fn new(
        world_address: FieldElement,
        account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    ) -> Self {
        Self {
            world_address,
            account: Arc::new(account),
            block_id: BlockId::Tag(BlockTag::Latest),
            provider: JsonRpcClient::new(HttpTransport::new(
                Url::parse(configs::JSON_RPC_ENDPOINT).unwrap(),
            )),
        }
    }
}

pub struct DojoPlugin;

impl Plugin for DojoPlugin {
    fn build(&self, app: &mut App) {
        let url = Url::parse(configs::JSON_RPC_ENDPOINT).unwrap();
        let account_address = FieldElement::from_str(configs::ACCOUNT_ADDRESS).unwrap();

        let account = SingleOwnerAccount::new(
            JsonRpcClient::new(HttpTransport::new(url)),
            LocalWallet::from_signing_key(SigningKey::from_secret_scalar(
                FieldElement::from_str(configs::ACCOUNT_SECRET_KEY).unwrap(),
            )),
            account_address,
            cairo_short_string_to_felt("KATANA").unwrap(),
            ExecutionEncoding::Legacy,
        );

        let world_address = FieldElement::from_str(configs::WORLD_ADDRESS).unwrap();

        // creating world and adding systems
        app.add_plugins(TokioTasksPlugin::default())
            // add events
            .add_event::<GameUpdate>()
            // resources
            .insert_resource(DojoEnv::new(world_address, account))
            // starting system
            .add_systems(
                Startup,
                (
                    setup,
                    initialise_world_thread,
                    interact_object_thread,
                    escape_game,
                ),
            )
            // update systems
            .add_systems(Update, sync_dojo_state);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(DojoSyncTime::from_seconds(configs::DOJO_SYNC_INTERVAL));
    commands.spawn(GameData { started: false });
}

#[derive(Component)]
struct DojoSyncTime {
    timer: Timer,
}

impl DojoSyncTime {
    fn from_seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
        }
    }
}

fn sync_dojo_state(
    mut dojo_sync_time: Query<&mut DojoSyncTime>,
    time: Res<Time>,
    spawn_room: Res<StartGameCommand>,

    mut game: Query<&mut GameData>,
) {
    let mut dojo_time = dojo_sync_time.single_mut();
    let mut game_state = game.single_mut();

    if dojo_time.timer.just_finished() {
        dojo_time.timer.reset();

        // if not spawn the player by calling the channel
        if game_state.started == false {
            if let Err(e) = spawn_room.try_send() {
                log::error!("Spawn room channel: {e}");
            }

            game_state.started = true;
        }
    } else {
        dojo_time.timer.tick(time.delta());
    }
}

fn initialise_world_thread(
    env: Res<DojoEnv>,
    runtime: ResMut<TokioTasksRuntime>,
    mut commands: Commands,
) {
    let (tx, mut rx) = mpsc::channel::<()>(8);
    commands.insert_resource(StartGameCommand(tx));

    let account = env.account.clone();

    let turns_remaining: u64 = 10;

    runtime.spawn_background_task(move |mut ctx| async move {
        while let Some(_) = rx.recv().await {
            match account
                .execute(vec![Call {
                    to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                    selector: get_selector_from_name("initialise").unwrap(),
                    calldata: vec![turns_remaining.into()],
                }])
                .send()
                .await
            {
                Ok(_) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Game Initialized.");
                    })
                    .await;
                }
                Err(e) => {
                    log::error!("Run create system: {e}");
                    println!("Error {}", e);
                }
            }
        }
    });
}

fn spawn_object_thread(
    env: Res<DojoEnv>,
    runtime: ResMut<TokioTasksRuntime>,
    mut commands: Commands,
) {
    let (tx, mut rx) = mpsc::channel::<(String, String)>(8);
    commands.insert_resource(SpawnObjectState(tx));

    let account = env.account.clone();

    let turns_remaining: u64 = 10;

    runtime.spawn_background_task(move |mut ctx| async move {
        while let Some((name, description)) = rx.recv().await {
            match account
                .execute(vec![Call {
                    to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                    selector: get_selector_from_name("spawn_object").unwrap(),
                    calldata: vec![
                        cairo_short_string_to_felt(&name).unwrap(),
                        cairo_short_string_to_felt(&description).unwrap(),
                    ],
                }])
                .send()
                .await
            {
                Ok(_) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Game Initialized.");
                    })
                    .await;
                }
                Err(e) => {
                    log::error!("Run create system: {e}");
                    println!("Error {}", e);
                }
            }
        }
    });
}

fn interact_object_thread(
    env: Res<DojoEnv>,
    runtime: ResMut<TokioTasksRuntime>,
    mut commands: Commands,
) {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    commands.insert_resource(InteractObjectState(tx));

    let account = env.account.clone();
    let world_address = env.world_address.clone();

    runtime.spawn_background_task(move |mut ctx| async move {
        while let Some(data) = rx.recv().await {
            let object_id = cairo_short_string_to_felt(&data).unwrap();
            match account
                .execute(vec![Call {
                    to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                    selector: get_selector_from_name("interact").unwrap(),
                    calldata: vec![object_id],
                }])
                .send()
                .await
            {
                Ok(tx) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Interaction with {}.", tx.transaction_hash);
                    })
                    .await;

                    tokio::spawn(async move {
                        thread::sleep(Duration::from_millis(250));
                        let schema =
                            fetch_schema(world_address, object_id, String::from("Object")).await;
                        if let Ty::Struct(struct_ty) = schema {
                            for child in struct_ty.children {
                                if child.name == "description" {
                                    println!("{} - {}", data, child.name);
                                    if let Ty::Primitive(Primitive::Felt252(Some(felt))) = child.ty
                                    {
                                        println!("{}", FieldElement::to_hex(felt));
                                    }
                                }
                            }
                        }

                        let schema =
                            fetch_schema(world_address, object_id, String::from("Game")).await;
                        if let Ty::Struct(struct_ty) = schema {
                            for child in struct_ty.children {
                                if child.name == "turns_remaining" {
                                    println!("{}", child.name);
                                    if let Ty::Primitive(Primitive::U64(Some(felt))) = child.ty {
                                        println!("{}", felt);
                                        println!("-----------");
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    log::error!("Run create system: {e}");
                    println!("Error {}", e);
                }
            }
        }
    });
}

async fn fetch_schema(world_address: FieldElement, object_id: FieldElement, model: String) -> Ty {
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse(configs::JSON_RPC_ENDPOINT).unwrap(),
    ));
    let world = WorldContractReader::new(world_address, provider);
    let position = world.model(&model).await.unwrap();

    if model == "Game" {
        let object_id_slice = &[FieldElement::from_hex_be(configs::ACCOUNT_ADDRESS).unwrap()];

        return position.entity(object_id_slice).await.unwrap();
    }
    let object_id_slice = &[
        FieldElement::from_hex_be(configs::ACCOUNT_ADDRESS).unwrap(),
        object_id,
    ];

    position.entity(object_id_slice).await.unwrap()
}

fn escape_game(env: Res<DojoEnv>, runtime: ResMut<TokioTasksRuntime>, mut commands: Commands) {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    commands.insert_resource(EscapeState(tx));

    let account = env.account.clone();
    let world_address = env.world_address.clone();
    let object_id = cairo_short_string_to_felt("dummy").unwrap();
    runtime.spawn_background_task(move |mut ctx| async move {
        while let Some(data) = rx.recv().await {
            let secret = cairo_short_string_to_felt(&data).unwrap();
            match account
                .execute(vec![Call {
                    to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                    selector: get_selector_from_name("escape").unwrap(),
                    calldata: vec![secret],
                }])
                .send()
                .await
            {
                Ok(tx) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Trying to escape at hash {}.", tx.transaction_hash);
                    })
                    .await;
                    tokio::spawn(async move {
                        thread::sleep(Duration::from_millis(250));
                        let schema =
                            fetch_schema(world_address, object_id, String::from("Game")).await;
                        if let Ty::Struct(struct_ty) = schema {
                            for child in struct_ty.children {
                                if child.name == "turns_remaining" || child.name == "is_finished" {
                                    println!("{}", child.name);
                                    if let Ty::Primitive(Primitive::U64(Some(felt))) = child.ty {
                                        println!("{}", felt);
                                    } else if let Ty::Primitive(Primitive::Bool(Some(value))) =
                                        child.ty
                                    {
                                        println!("{}", value);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    log::error!("Run create system: {e}");
                    println!("Error {}", e);
                }
            }
        }
    });
}
trait ToHex {
    fn to_hex(self) -> String;
}
impl ToHex for FieldElement {
    fn to_hex(self) -> String {
        format!("{:#x}", self)
    }
}
