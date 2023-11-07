use crate::configs;
use crate::resources::*;
use anyhow::Result;
use async_compat::Compat;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use dojo_types::{primitive::Primitive, schema::Ty};
use dojo_world::contracts::WorldContractReader;
use futures_lite::future;
use regex::Regex;
use std::thread;
use std::time::Duration;
use std::{str::FromStr, sync::Arc};
use url::Url;

use starknet::{
    accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount},
    core::{
        types::{BlockId, BlockTag, FieldElement},
        utils::{cairo_short_string_to_felt, get_selector_from_name, parse_cairo_short_string},
    },
    providers::jsonrpc::HttpTransport,
    providers::JsonRpcClient,
    signers::{LocalWallet, SigningKey},
};

#[derive(Resource)]
pub struct DojoEnv {
    block_id: BlockId,
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
            .add_systems(Startup, (setup, task_init, handle_task_spawn_object))
            // update systems
            .add_systems(
                Update,
                (
                    sync_dojo_state,
                    handle_task_init,
                    handle_task_interact,
                    handle_task_escape,
                ),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(DojoSyncTime::from_seconds(configs::DOJO_SYNC_INTERVAL));
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

fn sync_dojo_state(mut dojo_sync_time: Query<&mut DojoSyncTime>, time: Res<Time>) {
    let mut dojo_time = dojo_sync_time.single_mut();

    if dojo_time.timer.just_finished() {
        dojo_time.timer.reset();
    } else {
        dojo_time.timer.tick(time.delta());
    }
}

#[derive(Component)]
struct InitializeGame(Task<bool>);

fn task_init(mut commands: Commands, env: Res<DojoEnv>) {
    let account = env.account.clone();
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(Compat::new(async move {
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
    }));
    commands.spawn(InitializeGame(task));
}

fn handle_task_init(mut commands: Commands, mut game_task: Query<(Entity, &mut InitializeGame)>) {
    for (entity, mut task) in &mut game_task {
        if let Some(_) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<InitializeGame>();
        }
    }
}
enum ObjectData {
    TurnsRemaining(u64),
    ObjectDescription(String),
    IsFinished(bool),
}

#[derive(Component)]
struct InteractObject(Task<Vec<ObjectData>>);

pub fn task_interact(commands: &mut Commands, env: &Res<DojoEnv>, object_id: FieldElement) {
    let account = env.account.clone();
    let thread_pool = AsyncComputeTaskPool::get();
    let world_address = env.world_address.clone();

    let task = thread_pool.spawn(Compat::new(async move {
        let mut my_list: Vec<ObjectData> = Vec::new();
        match account
            .execute(vec![Call {
                to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                selector: get_selector_from_name("interact").unwrap(),
                calldata: vec![object_id],
            }])
            .send()
            .await
        {
            Ok(_) => {
                thread::sleep(Duration::from_millis(250));
                let schema = fetch_schema(world_address, object_id, String::from("Object")).await;

                if let Ty::Struct(struct_ty) = schema {
                    for child in struct_ty.children {
                        if child.name == "description" {
                            // println!("{} - {}", data, child.name);
                            if let Ty::Primitive(Primitive::Felt252(Some(felt))) = child.ty {
                                my_list
                                    .push(ObjectData::ObjectDescription(FieldElement::to_hex(felt)))
                            }
                        }
                    }
                }

                let schema = fetch_schema(world_address, object_id, String::from("Game")).await;
                if let Ty::Struct(struct_ty) = schema {
                    for child in struct_ty.children {
                        if child.name == "turns_remaining" {
                            // println!("{}", child.name);
                            if let Ty::Primitive(Primitive::U64(Some(felt))) = child.ty {
                                // println!("{}", felt);
                                // println!("-----------");
                                my_list.push(ObjectData::TurnsRemaining(felt));
                            }
                        }
                        if child.name == "is_finished" {
                            if let Ty::Primitive(Primitive::Bool(Some(felt))) = child.ty {
                                my_list.push(ObjectData::IsFinished(felt));
                            }
                        }
                    }
                }
                my_list
            }
            Err(e) => {
                println!("Error {}", e);
                let my_list: Vec<ObjectData> = Vec::new();
                my_list
            }
        }
    }));
    commands.spawn(InteractObject(task));
}

fn handle_task_interact(
    mut commands: Commands,
    mut game_task: Query<(Entity, &mut InteractObject)>,
    mut moves: ResMut<MovesRemaining>,
) {
    for (entity, mut task) in &mut game_task {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0)) {
            for obj in response {
                match obj {
                    ObjectData::TurnsRemaining(turns) => {
                        println!("Turns remaining: {}", turns);
                        moves.0 = turns;
                    }
                    ObjectData::ObjectDescription(text) => {
                        let felt = parse_felt_value(&text).unwrap();
                        let decoded = parse_cairo_short_string(&felt).unwrap();
                        println!("Object description: {decoded}");
                    }
                    ObjectData::IsFinished(_) => {}
                }
            }

            commands.entity(entity).remove::<InteractObject>();
        }
    }
}

#[derive(Component)]
struct EscapeGame(Task<Vec<ObjectData>>);

pub fn task_escape(commands: &mut Commands, env: &Res<DojoEnv>, secret: String) {
    let account = env.account.clone();
    let thread_pool = AsyncComputeTaskPool::get();
    let world_address = env.world_address.clone();

    let task = thread_pool.spawn(Compat::new(async move {
        let mut my_list: Vec<ObjectData> = Vec::new();
        match account
            .execute(vec![Call {
                to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                selector: get_selector_from_name("escape").unwrap(),
                calldata: vec![cairo_short_string_to_felt(&secret).unwrap()],
            }])
            .send()
            .await
        {
            Ok(_) => {
                thread::sleep(Duration::from_millis(250));
                let schema = fetch_schema(
                    world_address,
                    cairo_short_string_to_felt("Door").unwrap(),
                    String::from("Game"),
                )
                .await;

                if let Ty::Struct(struct_ty) = schema {
                    for child in struct_ty.children {
                        if child.name == "turns_remaining" {
                            if let Ty::Primitive(Primitive::U64(Some(felt))) = child.ty {
                                my_list.push(ObjectData::TurnsRemaining(felt));
                            }
                        } else if child.name == "is_finished" {
                            if let Ty::Primitive(Primitive::Bool(Some(felt))) = child.ty {
                                my_list.push(ObjectData::IsFinished(felt));
                            }
                        }
                    }
                }
                my_list
            }
            Err(e) => {
                println!("Error {}", e);
                let my_list: Vec<ObjectData> = Vec::new();
                my_list
            }
        }
    }));
    commands.spawn(EscapeGame(task));
}

fn handle_task_escape(
    mut commands: Commands,
    mut game_task: Query<(Entity, &mut EscapeGame)>,
    mut moves: ResMut<MovesRemaining>,
) {
    for (entity, mut task) in &mut game_task {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0)) {
            for obj in response {
                match obj {
                    ObjectData::ObjectDescription(_) => {}
                    ObjectData::TurnsRemaining(turns) => {
                        println!("Turns remaining: {}", turns);
                        moves.0 = turns;
                    }
                    ObjectData::IsFinished(value) => {
                        println!("{}", value);
                        if value {
                            println!("You have escaped the room!");
                        } else {
                            println!("Wrong secret. Try again.")
                        }
                    }
                }
            }
            commands.entity(entity).remove::<EscapeGame>();
        }
    }
}

#[derive(Component)]
struct TaskSpawnObject(Task<bool>);

pub fn task_spawn_object(
    commands: &mut Commands,
    env: &Res<DojoEnv>,
    objects_id: Vec<FieldElement>,
    objects_description: Vec<FieldElement>,
) {
    let account = env.account.clone();
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(Compat::new(async move {
        let mut calldata = Vec::new();

        // Add the length of each vector as the first element in calldata
        calldata.push(objects_id.len().into());
        calldata.extend(objects_id.iter().cloned());
        calldata.push(objects_description.len().into());
        calldata.extend(objects_description.iter().cloned());
        println!("{}", calldata.len());
        thread::sleep(Duration::from_millis(250));
        // ... concatenate elements from other vectors ...
        match account
            .execute(vec![Call {
                to: FieldElement::from_hex_be(configs::ACTIONS_ADDRESS).unwrap(),
                selector: get_selector_from_name("spawn_object").unwrap(),
                calldata: calldata,
            }])
            .send()
            .await
        {
            Ok(_) => {
                println!("Objects Spawned.");
                true
            }
            Err(e) => {
                println!("Error {}", e);
                false
            }
        }
    }));
    commands.spawn(TaskSpawnObject(task));
}

fn handle_task_spawn_object(
    mut commands: Commands,
    mut game_task: Query<(Entity, &mut TaskSpawnObject)>,
) {
    for (entity, mut task) in &mut game_task {
        if let Some(_) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<TaskSpawnObject>();
        }
    }
}

// used to get the schema
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

// to convert hex to string
trait ToHex {
    fn to_hex(self) -> String;
}
impl ToHex for FieldElement {
    fn to_hex(self) -> String {
        format!("{:#x}", self)
    }
}

pub fn parse_felt_value(felt: &str) -> Result<FieldElement> {
    let regex_dec_number = Regex::new("^[0-9]{1,}$").unwrap();

    if regex_dec_number.is_match(felt) {
        Ok(FieldElement::from_dec_str(felt)?)
    } else {
        Ok(FieldElement::from_hex_be(felt)?)
    }
}
