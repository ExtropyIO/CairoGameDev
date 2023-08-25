use crate::configs;
use bevy::ecs::system::SystemState;
use bevy::log;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tokio_tasks::TaskContext;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use dojo_client::contract::world::WorldContract;
use num::bigint::BigUint;
use num::{FromPrimitive, ToPrimitive};
use rand::Rng;
use starknet::accounts::SingleOwnerAccount;
use starknet::core::types::{BlockId, BlockTag, FieldElement};
use starknet::core::utils::cairo_short_string_to_felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::ops::Div;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

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
            LocalWallet::from_singning_key(SigningKey::from_secret_scalar(
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
            .add_startup_systems((setup))
            // update systems
            .add_system(sync_dojo_state);
    }
}

fn setup(mut commands: Comands) {
    commands.spawn(DojoSyncTime::from_seconds(configs::DOJO_SYNC_INTERVAL))
}

#[derive(Component)]
struct DojoSyncTime {
    time: Timer,
}

impl DojoSyncTime {
    fn from_seconds(duration: f32 ) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
        }
    }
}

fn sync_dojo_state(
    mut dojo_sync_time: Query<&mut DojoSyncTime>, time: Res<Time>) {
        // define tick logic
    }
)
