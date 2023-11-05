use crate::configs;
use crate::resources::*;
use bevy::{
    ecs::{event::Event, system::SystemState},
    log,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use hex;

use dojo_types::{primitive::Primitive, schema::Ty};
use dojo_world::contracts::WorldContractReader;

use futures_lite::future;
use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};
use std::{str::FromStr, sync::Arc};
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
        app
            // resources
            .insert_resource(DojoEnv::new(world_address, account))
            // starting system
            .add_systems(Startup, (setup, spawn_task))
            // update systems
            .add_systems(Update, (sync_dojo_state, handle_task));
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
    mut commands: Commands,
    env: Res<DojoEnv>,

    mut game: Query<&mut GameData>,
) {
    let mut dojo_time = dojo_sync_time.single_mut();
    let mut game_state = game.single_mut();

    if dojo_time.timer.just_finished() {
        dojo_time.timer.reset();

        if game_state.started == false {
            spawn_task(commands, env);

            game_state.started = true;
        }
    } else {
        dojo_time.timer.tick(time.delta());
    }
}

#[derive(Component)]
struct InitializeGame(Task<bool>);

fn spawn_task(mut commands: Commands, env: Res<DojoEnv>) {
    let account = env.account.clone();
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async move {
        println!("{}", account.address());
        let turns_remaining: u64 = 10;
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
                println!("Game Initialized.");
                true
            }
            Err(e) => {
                println!("Error {}", e);
                false
            }
        }
    });

    commands.spawn(InitializeGame(task));
}

fn handle_task(mut commands: Commands, mut game_task: Query<(Entity, &mut InitializeGame)>) {
    for (entity, mut task) in &mut game_task {
        if let Some(response) = future::block_on(future::poll_once((&mut task.0))) {
            println!("{}", response);
            commands.entity(entity).remove::<InitializeGame>();
        }
    }
}
