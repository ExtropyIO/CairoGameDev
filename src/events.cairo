#[event]
#[derive(Copy, Drop, starknet::Event)]
enum Event {
    ObjectData: ObjectData,
    GameState: GameState,
}

#[derive(Drop, Copy, Serde, starknet::Event)]
struct ObjectData {
    object_id: felt252,
    description: felt252,
}

#[derive(Drop, Copy, Serde, starknet::Event)]
struct GameState {
    game_state: felt252,
}

