# Room Escape - A Cairo Game

// Description of the game

## Setup

### Dojo instalation

Check out the instalation steps [here](https://book.dojoengine.org/).

### Bevy instalation

Check out the instalation steps [here](https://bevyengine.org/learn/book/getting-started/setup/).

### Deploying the world

First, get Katana started by running the following command:

```bash
katana --disable-fee
```

Build the contract:

```bash
sozo build
```

Deploy the contract:

```bash
sozo migrate --name room_escape
```

Once deployed, get the `WORLD_ADDRESS` and `ACTION` contracts and copy them into the `src/configs.rs`

```rust
// world
pub const WORLD_ADDRESS: &str = "YOUR_WORLD_CONTRACT_HERE";
pub const ACTIONS_ADDRESS: &str ="YOUR_ACTION_CONTRACT_HERE";
```

### Starting the game

Now that we have everything setup,

```bash
cargo run
```

## Game commands

Keyboard commands:

- `A` - move left
- `B` - move right
- `E` - interact with the object
