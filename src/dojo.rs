use crate::configs;
use crate::resources::{InteractObjectState, StartGameCommand};
use bevy::log;
use bevy::prelude::*;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use dojo_client::contract::world::WorldContract;
use starknet::accounts::SingleOwnerAccount;
use starknet::core::types::{BlockId, BlockTag, FieldElement};
use starknet::core::utils::cairo_short_string_to_felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

#[derive(Component)]
pub struct GameData {
    started: bool,
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
    let world_address = env.world_address;
    let block_id = env.block_id;

    let start_time: u64 = 123455;
    let turns_remaining: u64 = 10;
    println!("HERE01");
    runtime.spawn_background_task(move |mut ctx| async move {
        let world = WorldContract::new(world_address, account.as_ref());
        let start_game_system = world.system("create", block_id).await.unwrap();
        println!("HERE02");
        while let Some(_) = rx.recv().await {
            println!("HERE03");
            match start_game_system
                .execute(vec![start_time.into(), turns_remaining.into()])
                .await
            {
                Ok(_) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Game spawned");
                    })
                    .await;
                }
                Err(e) => {
                    log::error!("Run spawn_object system: {e}");
                    println!("Error {}", e);
                }
            }
        }
        println!("Start Dojo Loop");
    });
}

fn interact_object_thread(
    env: Res<DojoEnv>,
    runtime: ResMut<TokioTasksRuntime>,
    mut commands: Commands,
) {
    let (tx, mut rx) = mpsc::channel::<()>(8);
    commands.insert_resource(InteractObjectState(tx));

    let account = env.account.clone();
    let world_address = env.world_address;
    let block_id = env.block_id;

    let game_id: u32 = 1;

    runtime.spawn_background_task(move |mut ctx| async move {
        let world = WorldContract::new(world_address, account.as_ref());
        let interact_system = world.system("interact", block_id).await.unwrap();

        while let Some(_) = rx.recv().await {
            println!("Interact System - Triggered");
            match interact_system
                .execute(vec![
                    game_id.into(),
                    FieldElement::from_str("0x5061696e74696e67").unwrap(),
                ])
                .await
            {
                Ok(data) => {
                    ctx.run_on_main_thread(move |_ctx| {
                        println!("Object interacted");
                        println!("{}", data.transaction_hash);
                    })
                    .await;
                }
                Err(e) => {
                    log::error!("Run spawn_object system: {e}");
                    println!("Error {}", e);
                }
            }
        }
        println!("Start Dojo Loop");
    });
}
