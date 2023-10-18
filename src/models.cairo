use array::ArrayTrait;
use core::debug::PrintTrait;
use starknet::ContractAddress;


#[derive(Model, Copy, Drop, Serde)]
struct Game {
    #[key]
    player: ContractAddress,
    game_id: u32,
    start_time: u64,
    turns_remaining: u64,
    is_finished: bool,
}


#[derive(Model, Copy, Drop, Serde)]
struct Object {
    #[key]
    game_id: u32,
    #[key]
    player: ContractAddress,
    #[key]
    object_id: felt252,
    description: felt252,
}

#[derive(Model, Copy, Drop, Serde)]
struct Door {
    #[key]
    game_id: u32,
    #[key]
    player_id: ContractAddress,
    secret: felt252,
}


#[generate_trait]
impl ObjectImpl of ObjectTrait {
    fn all() -> Span<felt252> {
        let mut objects = array!['Painting', 'Foto', 'Book', 'Strange Amulet',];
        objects.span()
    }
}


#[generate_trait]
impl GameImpl of GameTrait {
    #[inline(always)]
    fn tick(self: Game) -> bool {
        let info = starknet::get_block_info().unbox();

        if info.block_timestamp < self.start_time {
            return false;
        }
        if self.is_finished {
            return false;
        }
        true
    }
}
