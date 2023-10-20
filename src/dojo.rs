use crate::configs;
use crate::resources::*;
use bevy::ecs::event::Event;

use bevy::log;
use bevy::prelude::*;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};

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
        chain_id,
        types::{BlockId, BlockTag, FieldElement},
        utils::get_selector_from_name,
    },
    providers::SequencerGatewayProvider,
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
            // resources
            .insert_resource(DojoEnv::new(world_address, account))
            // starting system
            .add_systems(
                Startup,
                (setup, spawn_object_thread, interact_object_thread),
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
    interacted_object: Res<ObjectNameInteraction>,
) {
    let (tx, mut rx) = mpsc::channel::<String>(8);
    commands.insert_resource(InteractObjectState(tx));

    let account = env.account.clone();
    let mut object_id: FieldElement = FieldElement::from_str("").unwrap();

    runtime.spawn_background_task(move |mut ctx| async move {
        while let Some(data) = rx.recv().await {
            println!("{}", data);
            match FieldElement::from_str(&data.clone()) {
                Ok(result) => {
                    object_id = result;
                }
                Err(_) => {}
            }
            match account
                .execute(vec![Call {
                    to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                    selector: get_selector_from_name("interact").unwrap(),
                    calldata: vec![object_id],
                }])
                .send()
                .await
            {
                Ok(data) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Interaction with {}.", data.transaction_hash);
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
