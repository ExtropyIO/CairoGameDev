use dojo::world::{IWorldDispatcher, IWorldDispatcherTrait};
use dojo_examples::models::{Game, GameTrait, Object, ObjectTrait, Door};
use starknet::{ContractAddress, ClassHash};

#[starknet::interface]
trait IActions<TContractState> {
    fn spawn(self: @TContractState, turns_remaining: u64);
    fn interact(self: @TContractState, object_id: felt252);
    fn escape(self: @TContractState, secret: felt252);
    fn test(self: @TContractState);
}

#[dojo::contract]
mod actions {
    use super::IActions;
    use starknet::{ContractAddress, get_caller_address, get_block_timestamp};
    use dojo_examples::models::{Game, GameTrait, Object, ObjectTrait, Door};
    #[event]
    use dojo_examples::events::{Event, ObjectData, GameState};

    // impl: implement functions specified in trait
    #[external(v0)]
    impl ActionsImpl of IActions<ContractState> {
        fn test(self:@ContractState) {
            let world = self.world_dispatcher.read();
        }
        fn spawn(self: @ContractState, turns_remaining: u64) {
            // Access the world dispatcher for reading.
            let world = self.world_dispatcher.read();

            // Get the address of the current caller, possibly the player's address.
            let player = get_caller_address();

            let game_id = world.uuid();

            let start_time = get_block_timestamp();

            let game = Game {
                game_id, start_time, turns_remaining, is_finished: false, player: player,
            };

            let door = Door { game_id, player_id: player, secret: '1984', };

            set!(world, (game, door));

            set!(
                world,
                (
                    Object {
                        game_id: game_id,
                        player: player,
                        object_id: 'Painting',
                        description: 'An intriguing painting.',
                    },
                    Object {
                        game_id: game_id,
                        player: player,
                        object_id: 'Foto',
                        description: 'An egyptian cat.',
                    },
                    Object {
                        game_id: game_id,
                        player: player,
                        object_id: 'Strange Amulet',
                        description: 'Until tomorrow',
                    },
                    Object {
                        game_id: game_id,
                        player: player,
                        object_id: 'Book',
                        description: '1984',
                    },
                )
            );

            emit!(world, GameState { game_state: 'Game Initialized' });
        }

        fn interact(self: @ContractState, object_id: felt252) {
            // Access the world dispatcher for reading.
            let world = self.world_dispatcher.read();

            // Get the address of the current caller, possibly the player's address.
            let player = get_caller_address();

            let mut game = get!(world, player, (Game));

            // can assert if game exists for the player 
            assert(game.tick(), 'Cannot Progress');

            if game.turns_remaining == 0 {
                emit!(world, GameState { game_state: 'Game Over' });
                return ();
            } else {
                game.turns_remaining -= 1;
            }

            let object = get!(world, (player, object_id).into(), Object);

            set!(world, (game,));

            // emit item data
            emit!(world, GameState { game_state: 'Checking Item' });
            emit!(
                world, ObjectData { object_id: object.object_id, description: object.description }
            );
        }

        fn escape(self: @ContractState, secret: felt252) {
            // Access the world dispatcher for reading.
            let world = self.world_dispatcher.read();

            // Get the address of the current caller, possibly the player's address.
            let player = get_caller_address();

            let mut game = get!(world, player, (Game));

            assert(game.tick(), 'Cannot Progress');

            if game.turns_remaining == 0 {
                emit!(world, GameState { game_state: 'Game Over' });
                return ();
            } else {
                game.turns_remaining -= 1;
            }

            let door = get!(world, player, (Door));

            if door.secret == secret {
                game.is_finished = true;
                set!(world, (game,));

                emit!(world, GameState { game_state: 'Escaped' });
            }

            set!(world, (game,));

            emit!(world, GameState { game_state: 'Wrong Secret' });
        }
    }
}
