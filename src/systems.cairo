#[system]
mod create {
    use array::ArrayTrait;
    use box::BoxTrait;
    use traits::Into;
    use dojo::world::Context;
    use starknet::{ContractAddress};

    use dojo_examples::components::{Door};
    use dojo_examples::components::{Object, ObjectTrait};
    use dojo_examples::components::{Game, GameTrait};

    #[event]
    use dojo_examples::events::{Event, ObjectData, GameState};

    fn execute(ctx: Context, start_time: u64, turns_remaining: usize,) {

        let game_id = ctx.world.uuid();

        let game = Game {
            game_id,
            start_time,
            turns_remaining,
            is_finished: false,
            creator: ctx.origin,
        };

        let door = Door {
            game_id,
            player_id: ctx.origin,
            secret: '1984',
        };

        set !(ctx.world, (game, door));

        set !(
            ctx.world,
            (
                Object {
                    game_id:game_id, player_id: ctx.origin, object_id: 'Painting', description: 'An intriguing painting.', 
                    },
                Object {
                    game_id:game_id, player_id: ctx.origin, object_id: 'Foto', description: 'An egyptian cat.',
                    },
                Object {
                    game_id:game_id, player_id: ctx.origin, object_id: 'Strange Amulet', description: 'Until tomorrow',
                    },
                Object {
                    game_id:game_id, player_id: ctx.origin, object_id: 'Book', description: '1984',
                    },
            )
        );

        emit!(ctx.world, GameState { game_state: 'Game Initialized'});
        
        return ();
    }
}


#[system]
mod interact {
    use array::ArrayTrait;
    use box::BoxTrait;
    use traits::Into;
    use dojo::world::Context;

    use dojo_examples::components::{Object, ObjectTrait};
    use dojo_examples::components::{Game, GameTrait};

    #[event]
    use dojo_examples::events::{Event, ObjectData, GameState};

    fn execute(ctx: Context, game_id: u32, object_id: felt252) {

        let player_id = ctx.origin;

        let mut game = get!(ctx.world, game_id, (Game));

        assert(game.tick(), 'Cannot Progress');

        if game.turns_remaining == 0 {
            emit!(ctx.world, GameState {game_state: 'Game Over'});
            return ();
        } else {
            game.turns_remaining -= 1;
        }

        let object = get! (ctx.world, (game_id, player_id, object_id).into(), Object);

        set!(ctx.world, (game, ));
        
        // emit item data
        emit!(ctx.world, GameState { game_state: 'Checking Item'});
        emit!(ctx.world, ObjectData { object_id: object.object_id, description: object.description });
    }
}

#[system]
mod escape {
    use array::ArrayTrait;
    use box::BoxTrait;
    use traits::Into;
    use dojo::world::Context;

    use dojo_examples::components::{Door};
    use dojo_examples::components::{Object, ObjectTrait};
    use dojo_examples::components::{Game, GameTrait};


    #[event]
    use dojo_examples::events::{Event, ObjectData, GameState};

    fn execute(ctx: Context, game_id: u32, secret: felt252)  {

        let player_id = ctx.origin;

        let mut game = get!(ctx.world, game_id, Game);

        assert(game.tick(), 'Cannot Progress');

        if game.turns_remaining == 0 {
            emit!(ctx.world, GameState {game_state: 'Game Over'});
            return ();
        } else {
            game.turns_remaining -= 1;
        }
       
        let door = get! (ctx.world, (game_id, player_id).into(), Door);

        if door.secret == secret {

            game.is_finished = true;
            set !(ctx.world, (game, ));

            emit!(ctx.world, GameState {game_state: 'Escaped'});
        } 

        set !(ctx.world, (game, ));

        emit!(ctx.world, GameState {game_state: 'Wrong Secret'});

    }
}