---
title: Project Report
slug: /
sidebar:
  order: 0
---

## Introduction

Features that are / going to be implemented in RustMine include:

- Moving around the world
- Breaking-placing different block types (e.g., stone, dirt, wood)
- Saving and loading worlds
- Procedural ("infinite") world generation
- Different biomes
- Secure multiplayer

The project uses modern Rust gpu rendering instruments (wgpu).

## Installation and Usage

Run `cargo run --release --bin rustmine` or download the correct release binary for your system
(running the game in development mode is painfully slow).

Keybindings in the game:

- WASD or arrow keys: Move around
- Space: Jump / ascent (in spectator mode)
- Shift: Descent (in spectator mode)
- Left Control: Sprint
- 1–9: Change a block type
- Left mouse button: Break block
- Right mouse button: Place block
- G: Toggle flight mode
- ESC:
    - Main Menu: Exit the game
    - Pause Menu: Resume the game
    - In-Game: Open the pause menu
    - Singleplayer and Multiplayer: Back to the Main Menu
    - World Creation Menu: Back to the Singleplayer Menu

When creating a new world, the creation screen lets you set a world name and a seed,
which determines the procedurally generated terrain.
Worlds are listed on the main menu and can be loaded by selecting them.
While in-game, opening the pause menu lets you save the current world at any time.

## Challenges, Solutions, Lessons-Learned

What was particularly difficult and how was it solved?

Setting up the CI/CD pipeline was unexpectedly time-consuming. Validating that the pipeline behaved correctly end-to-end
required many iteration cycles — small mistakes
only surfaced after a full pipeline run, making the feedback loop slow.
The wgpu rendering stack was the other major technical hurdle. The API is low-level and demands a solid understanding of
the GPU pipeline before anything appears on screen. Buffer management, render passes, bind groups, and WGSL shaders all
had to come together correctly at once. Getting the initial renderer working took significant time, but the resulting
understanding of the graphics pipeline paid off throughout the rest of the project.

What was particularly instructive or useful?

Working directly with wgpu gave the team a much deeper understanding of the modern GPU pipeline than higher-level
engines would have allowed.
Managing buffer lifetimes, understanding render passes, and writing WGSL shaders from scratch were all valuable
low-level skills.
On the organizational side, using GitLab issues and milestones to track features kept the workload visible and made it
easier to distribute tasks across the team.

What would we do differently next time?

We would implement a chunk-based system for world generation and world culling earlier in the project.
That would allow us to generate the world and render the game in a more efficient manner and would save much more time
on the rewriting of the rendering code.
We would define a formal state machine architecture from the start. Initially, all game logic was consolidated into a
single state, which made the code increasingly difficult to extend and maintain. Introducing a TState trait with
distinct implementations such as GameState, MenuState, and others significantly improved modularity – but retrofitting
this structure onto existing code was more costly than designing it in from the beginning would have been.
