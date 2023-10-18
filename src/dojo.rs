use crate::resources::*;
use crate::{configs, MovesRemaining};
use bevy::ecs::event::{Event, Events};
use bevy::ecs::system::SystemState;
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
        );

        let world_address = FieldElement::from_str(configs::WORLD_ADDRESS).unwrap();

        // creating world and adding systems
        app.add_plugins(TokioTasksPlugin::default())
            // add events
            .add_event::<GameUpdate>()
            .add_event::<CheckGame>()
            // resources
            .insert_resource(DojoEnv::new(world_address, account))
            // starting system
            .add_systems(
                Startup,
                (
                    setup,
                    spawn_object_thread,
                    interact_object_thread,
                    fetch_component,
                ),
            )
            // update systems
            .add_systems(Update, (sync_dojo_state, check_game_update));
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

    let turns_remaining: u64 = 10;

    runtime.spawn_background_task(move |mut ctx| async move {
        let world = WorldContract::new(world_address, account.as_ref());
        let start_game_system = world.system("spawn", block_id).await.unwrap();

        while let Some(_) = rx.recv().await {
            match start_game_system
                .execute(vec![turns_remaining.into()])
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
                    FieldElement::from_str("0x5061696e74696e67").unwrap(), // object to interact with
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

fn fetch_component(env: Res<DojoEnv>, runtime: ResMut<TokioTasksRuntime>, mut commands: Commands) {
    let (tx, mut rx) = mpsc::channel::<()>(16);

    commands.insert_resource(CheckGame(tx));

    let account = env.account.clone();
    let world_address = env.world_address;
    let block_id = env.block_id;
    let player = FieldElement::from_str(configs::ACCOUNT_ADDRESS).unwrap();

    runtime.spawn_background_task(move |mut ctx| async move {
        let world = WorldContract::new(world_address, account.as_ref());

        let game_component = world.component("Game", block_id).await.unwrap();

        while let Some(_) = rx.recv().await {
            match game_component
                .entity(FieldElement::ZERO, vec![player], block_id)
                .await
            {
                Ok(update) => {
                    ctx.run_on_main_thread(move |ctx| {
                        println!("getting the component game");
                        // Create a new system state for an event writer associated with game updates.
                        let mut state: SystemState<EventWriter<GameUpdate>> =
                            SystemState::new(ctx.world);

                        // Retrieve a mutable reference to the event writer.
                        let mut update_game: EventWriter<'_, GameUpdate> = state.get_mut(ctx.world);

                        // Use the event writer to send a new game update event.
                        update_game.send(GameUpdate {
                            game_update: update,
                        })
                    })
                    .await;
                }

                Err(e) => {
                    log::error!("Query `Game` component: {e}");
                }
            }
        }
    });
}

// this is the function that reads the event from above
fn check_game_update(
    mut events: EventReader<GameUpdate>, //gets an event call
    mut query: Query<&mut Game>,
) {
    for e in events.iter() {
        //loop through every event
        println!("Game Events Update:");
        println!("{}", e.game_update[0]);
    }
}
