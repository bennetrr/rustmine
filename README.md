# <img height="50" src="rustmine/src/assets/icons/rustmine-icon.png" width="50"/> RustMine

## Overview

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

## Development Setup

1. Install [mise](https://mise.jdx.dev/installing-mise.html) (package manager for development tools)
2. Run `mise install`
3. If using RustRover: Configure IDE settings
   - Install the [_Mise_](https://plugins.jetbrains.com/plugin/24904-mise) plugin
   - In _Settings_ > _Rust_ > _Rustfmt_, check _Use Rustfmt instead of the built-in formatter_
   - In _Settings_ > _Tools_ > _Actions on Save_, check _Reformat Code_
   - In _Settings_ > _Rust_ > _External Linters_, check _Run external linter on the fly_
     and select _Clippy_ from the _External tool_ dropdown
   - Optional: Install the [_WGSL Support_](https://plugins.jetbrains.com/plugin/18110-wgsl-support) plugin

### Conventional Commits

This project uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) to determine the version for new releases:

- Prefix commit messages with a type
  - `feat!: ` or `fix!: ` indicate a breaking change
  - `feat: ` indicates a new feature
  - `fix: ` indicates a bug fix
  - `chore: `, `test: `, or `docs: ` indicate other changes
- Example: `feat: Add tree generation`, `chore: Refactor game state`, `chore: Format code`
- Optionally, a scope can be specified (e.g., `fix(ui): Fix misaligned buttons`)

Conventional commits are enforced through git hooks.

### Commands

- `cargo build`: Compiles the project
- `cargo run`: Runs the project
- `mise run format`: Formats the code
- `cargo clippy`: Lints the code (add `--fix` to automatically fix lints)
