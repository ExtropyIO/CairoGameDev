#[derive(Drop, Clone, Serde, PartialEq, starknet::Event)]
struct ObjectData {
    object_id: felt252,
    description: felt252,
}

#[derive(Drop, Clone, Serde, PartialEq, starknet::Event)]
struct GameState {
    game_state: felt252,
}

#[derive(Drop, Clone, Serde, PartialEq, starknet::Event)]
enum Event{
    ObjectData: ObjectData,
    GameState: GameState,
}

