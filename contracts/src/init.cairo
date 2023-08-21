#[system]
mod spawn_room {

    use array::ArrayTrait;
    use box::BoxTrait;
    use traits::Into;
    use dojo::world::Context;

    
    use game_demo::components::{Door, Table, Book};

    fn execute(ctx: Context) {

        set !(ctx.world, 
                (
                    Door { player: ctx.origin, locked: true,},
                    Table { player: ctx.origin, note: '1993',
                            // book: Book { player: ctx.origin, title: 'Fahrenheit 451'}
                            },
                )
            );


        return ();
    }
}


// #[system]
// mod get_item {

//     fn execute(ctx: Context, VAR: u16) {

//     }
// }


// #[system]
// mod drop_item {

//     fn execute(ctx: Context, VAR: u16) {

//     }
// }


// #[system]
// mod use_item {

//     fn execute(ctx: Context, VAR: u16) {

//     }
// }