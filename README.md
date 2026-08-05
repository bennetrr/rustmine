# <img height="50" src="rustmine/src/assets/icons/rustmine-icon.png" width="50"/> RustMine

Features: 

- Moving around the world
- Different block types (e.g., stone, dirt, wood)
- Breaking / placing blocks
- Saving and loading worlds
- Procedural ("infinite") world generation
- Different biomes
- Secure multiplayer with authentication

The project uses modern Rust gpu rendering instruments (wgpu).

Read the full project documentation [here](https://rustmine.bennet.ranft.ing/).

## Commands

- `cargo build`: Compiles the project
- `cargo run`: Runs the project
- `cargo fmt`: Formats the code (add `--check` to only print formatting errors)
- `cargo clippy`: Lints the code (add `--fix` to automatically fix lints)

## IDE Setup

### RustRover

1. Open _Settings_ > _Rust_ > _Rustfmt_ and check _Use Rustfmt instead of the built-in formatter_
2. Open _Settings_ > _Tools_ > _Actions on Save_ and check _Reformat Code_
3. Open _Settings_ > _Rust_ > _External Linters_, check _Run external linter on the fly_ and select _Clippy_ from the
   _External tool_ dropdown
