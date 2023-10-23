use crate::configs;
use crate::resources::*;

use bevy::ecs::event::Event;

use bevy::log;
use bevy::prelude::*;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use dojo_world::contracts::WorldContractReader;

use starknet::core::utils::cairo_short_string_to_felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;

use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::mpsc;
use url::Url;

use starknet::{
    accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount},
    core::{
        types::{BlockId, BlockTag, FieldElement},
        utils::get_selector_from_name,
    },
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
    pub game_update: Vec<FieldElement>, // vector of field elements
}

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
                    spawn_object_thread,
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

fn spawn_object_thread(
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
                    selector: get_selector_from_name("spawn").unwrap(),
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
                        fetch_schema(world_address, object_id).await;
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

async fn fetch_schema(world_address: FieldElement, object_id: FieldElement) {
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse(configs::JSON_RPC_ENDPOINT).unwrap(),
    ));
    let world = WorldContractReader::new(world_address, provider);
    let position = world.model("Object").await.unwrap();

    let object_id_slice = &[
        FieldElement::from_hex_be(configs::ACCOUNT_ADDRESS).unwrap(),
        object_id,
    ];

    dbg!(object_id_slice);

    match position.entity(object_id_slice).await {
        Ok(data) => {
            println!("{}", data);
        }
        Err(_) => {}
    }
}

fn escape_game(env: Res<DojoEnv>, runtime: ResMut<TokioTasksRuntime>, mut commands: Commands) {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    commands.insert_resource(EscapeState(tx));

    let account = env.account.clone();

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
                }
                Err(e) => {
                    log::error!("Run create system: {e}");
                    println!("Error {}", e);
                }
            }
        }
    });
}
