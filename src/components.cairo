use array::ArrayTrait;
use starknet::ContractAddress;

#[derive(Component, Copy, Drop, Serde, SerdeLen)]
struct Door {
    #[key]
    player: ContractAddress,
    locked: bool,
}

#[derive(Component, Copy, Drop, Serde, SerdeLen)]
struct Table {
    #[key]
    player: ContractAddress,
    note: felt252,
    // book: Book,
}

#[derive(Component, Copy, Drop, Serde, SerdeLen)]
struct Book{
    #[key]
    player: ContractAddress,
    title: felt252, 
}

#[derive(Component, Copy, Drop, Serde, SerdeLen)]
struct Backpack{
    #[key]
    player: ContractAddress,
    key: bool,
    note: bool, 
}
