---
title: Use Cases
sidebar:
  order: 1
---

## Release 1

### UC1: Player Movement

**ID**: UC1<br>
**Name**: Player Movement<br>
**Description**: Navigating around the world using the mouse and keyboard<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses `W` or `⬆` on the keyboard
- System moves the camera forwards
- Player presses `A` or `⬅` on the keyboard
- System moves the camera to the left
- Player presses `S` or `⬇` on the keyboard
- System moves the camera backwards
- Player presses `D` or `⮕` on the keyboard
- System moves the camera to the right
- Player presses `SPACE` on the keyboard while on the ground
- System moves the camera up
- Player moves the mouse
- System moves the camera angle

**Post-conditions**: n/a

### UC2: Spectator Mode

**ID**: UC2<br>
**Name**: Spectator Mode<br>
**Description**: Disabling player-block collisions and gravity, allowing the player to move freely in the world<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses `G` on the keyboard
- System enables spectator mode
- Player moves around (→ UC1)
- System does not enforce player-block collisions and gravity
- Player presses `SPACE` on the keyboard
- System moves camera up
- Player presses `LEFT SHIFT` on the keyboard
- System moves camera down
- Player presses `G` on the keyboard again
- System disables spectator mode

**Post-conditions**: n/a

### UC3: Selecting a Block Type

**ID**: UC3<br>
**Name**: Selecting a Block Type<br>
**Description**: Selecting a block type to place in the world (→ UC4)<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses one of `1`, `2`, `3`, `4`, `5`, `6`, `7`, `8`, `9` on the keyboard
- System selects the block type corresponding to the pressed key
    - `1` selects the block type “Grass”
    - `2` selects the block type “Dirt”
    - `3` selects the block type “Cobblestone”
    - `4` selects the block type “Oak Log”
    - `5` selects the block type “Spruce Log”
    - `6` selects the block type “Birch Log”
    - `7` selects the block type “Oak Leaves”
    - `8` selects the block type “Spruce Leaves”
    - `9` selects the block type “Birch Leaves”

**Post-conditions**:

- The block type corresponding to the pressed key is selected

### UC4: Placing Blocks

**ID**: UC4<br>
**Name**: Placing Blocks<br>
**Description**: Placing a block at the position the camera is pointing at<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses the right mouse button
- System places a block of the type selected by the player (→ UC3) or a default of “Grass” at the position the camera is
  pointing at
    - **Exception Flow**: The camera does not point at an existing block or the block is more than 5 blocks from the
      camera away
        - Do nothing

**Post-conditions**: n/a

### UC5: Destroying Blocks

**ID**: UC5<br>
**Name**: Destroying Blocks<br>
**Description**: Destroying the block at the position the camera is pointing at<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses the left mouse button
- System destroys the block at the position the camera is pointing at
    - **Exception Flow**: The camera does not point at an existing block or the block is more than 5 blocks from the
      camera away
        - Do nothing

**Post-conditions**: n/a

### UC6v1: Starting the Game

**ID**: UC6 (v1)<br>
**Name**: Starting the Game<br>
**Description**: Creates a new world with procedurally generated blocks with a fixed size of 16 × 16 chunks (256 × 256
blocks)<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**: n/a

**Steps**:

- Player starts the game
- System generates a new world with a fixed size of 16 × 16 chunks (256 × 256 blocks)
- System shows the game

**Post-conditions**:

- Player has loaded a world

## Release 2

### UC6v2: Creating a New World

**ID**: UC6 (v2)<br>
**Name**: Creating a New World<br>
**Description**: Creates a new world with procedurally generated blocks with a fixed size of 16 × 16 chunks (256 × 256
blocks)<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**: n/a

**Steps**:

- Player starts the game
- System shows main menu
- Player presses “Singleplayer” button
- System shows singleplayer world selection menu
- Player presses “Create New World” button
- System shows world creation form
- Player enters a world name
- Player optionally enters a seed
- Player presses “Create World” button
- System generates a new world with a fixed size of 16 × 16 chunks (256 × 256 blocks)
    - **Exception Flow**: Name input field is empty
        - System shows an error message
        - Continue with “Player enters a world name”
    - **Exception Flow**: There exists already a world with the entered name
        - System shows an error message
        - Continue with “Player enters a world name”
    - **Exception Flow**: Seed input field is empty
        - A randomly generated number is used as the seed
        - Continue with “System generates a new world…”
- System shows the game

**Post-conditions**:

- Player has at least one saved world
- Player has loaded a world

### UC7: Loading a Saved World

**ID**: UC7<br>
**Name**: Loading a Saved World<br>
**Description**: Loads a world previously saved to disk<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has at least one saved world

**Steps**:

- Player starts the game
- System shows main menu
- Player presses “Singleplayer” button
- System shows singleplayer world selection menu
- Player selects a world from the list
- System highlights the selected world
- Player presses “Play Selected World” button
- System loads the selected world from disk
- System shows the game

**Post-conditions**:

- Player has loaded a world

### UC8: Saving and Quitting the Loaded World

**ID**: UC8<br>
**Name**: Saving and Quitting the Loaded World<br>
**Description**: Saves the world that is currently played to disk and quits to the main menu<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player presses `ESC` on the keyboard
- System shows pause menu
- Player presses “Save and Quit to Menu”
- Systems saves world data to disk
- System shows main menu

**Post-conditions**: n/a

### UC9: Exiting the Pause Menu

**ID**: UC9<br>
**Name**: Exiting the Pause Menu<br>
**Description**: Going back from the pause menu to the game<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world
- Player is in the pause menu

**Steps**:

- Player presses `ESC` on the keyboard or presses the “Back to Game” button
- System shows the game

**Post-conditions**: n/a

### UC10: Quitting the Game

**ID**: UC10<br>
**Name**: Quitting the Game<br>
**Description**: Quitting the game from the main menu<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is in the main menu

**Steps**:

- Player presses `ESC` on the keyboard or presses the “Quit Game” button
- System exits

**Post-conditions**: n/a

### UC11: Aiming with the Crosshair

**ID**: UC11<br>
**Name**: Aiming with the Crosshair<br>
**Description**: A crosshair is shown at the center of the screen to indicate where the player is aiming at<br>
**Area**: In-game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- System displays a crosshair at the center of the screen
- Player moves the camera to look around the world (→ UC1)

**Post-conditions**: n/a

### NFR1: Performance

**ID**: NFR1<br>
**Goal**: The system should run with high performance<br>
**Area**: In-Game

**Questions / Metrics**:

- How fast is the rendering process?
    - Average frames per second on the deployment server
        - Minimum: 20 FPS
        - Target: 60 FPS
    - Max frame time (no frames slower than) on the deployment server
        - Minimum: 60 ms
        - Target: 20 ms
- How many invisible blocks are rendered?
    - Percent of blocks rendered outside the FOV from all rendered blocks
        - Minimum: 30 %
        - Target: 15 %
- How much memory does the game allocate?
    - Amount of allocated RAM after 15 minutes of playing
        - Minimum: 1 GB
        - Target: 600 MB
    - Amount of allocated VRAM after 15 minutes of playing
        - Minimum: 300 MB
        - Target: 150 MB
- How responsive is the system?
    - Input latency
        - Minimum: 40 ms
        - Target: 20 ms

### NFR2: Graphics Quality

**ID**: NFR2<br>
**Goal**: The graphics quality should be good<br>
**Area**: In-Game

**Questions / Metrics**:

- How do players rate the visual quality of distant textures?
    - Mean Opinion Score from 5 testers comparing mipmapped vs. non-mipmapped clips on a scale from 1 to 5
        - Minimum: 3.5
        - Target: 4.0
    - A/B preference: Preference of testers preferring the version with mipmapping
        - Minimum: 80 %
        - Target: 90 %

## Release 3

### UC6v3: Creating a New World

**ID**: UC6 (v3)<br>
**Name**: Creating a New World<br>
**Description**: Creates a new world and generates an initial area of 16 × 16 chunks (256 × 256 blocks)<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is logged in
- Player is in the main menu

**Steps**:

- Player presses “Singleplayer” button
- System shows singleplayer world selection menu
- Player presses “Create New World” button
- System shows world creation form
- Player enters a world name and optionally a seed
- Player presses “Create World” button
- System creates a new world and generates an initial area of 16 × 16 chunks (256 × 256 blocks)
    - **Exception Flow**: Name input field is empty
        - System shows an error message
        - Continue with “Player enters a world name”
    - **Exception Flow**: There exists already a world with the entered name
        - System shows an error message
        - Continue with “Player enters a world name”
    - **Exception Flow**: Seed input field is empty
        - A randomly generated number is used as the seed
        - Continue with “System generates a new world”

**Post-conditions**:

- Player has at least one saved world
- Player has loaded a world

### UC11: Infinite Procedural Generation

**ID**: UC11<br>
**Name**: Infinite Procedural Generation<br>
**Description**: While moving around the world, new chunks are generated<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- Player moves around in the world (→ UC1)
- System generates new chunks with the same seed used for the initial generation

**Post-conditions**: n/a

### UC12: Coordinate Overlay

**ID**: UC12<br>
**Name**: Coordinate Overlay<br>
**Description**: Show an overlay with the coordinates of the camera in the top left corner<br>
**Area**: In-Game<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world

**Steps**:

- System shows an overlay with the coordinates of the camera in the top left corner
- Player moves around in the world (→ UC1)
- System updates the overlay with the new coordinates of the camera

**Post-conditions**: n/a

### UC13: Deleting a Saved World

**ID**: UC13<br>
**Name**: Deleting a Saved World<br>
**Description**: Deletes a world previously saved to disk<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is in the main menu
- Player has at least one saved world

**Steps**:

- Player presses “Singleplayer” button
- System shows singleplayer world selection menu
- Player selects a world from the list
- System highlights the selected world
- Player presses “Delete Selected World” button
- System deletes the selected world from disk
- System shows the game

**Post-conditions**: n/a

## Further Development

### UC14: Signing up for a Player Account

**ID**: UC13<br>
**Name**: Signing up for a Player Account<br>
**Description**: Allowing the player to create an account with a unique player name<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**: n/a

**Steps**:

- Player starts the game
- System shows login screen
- Player presses “I don't have an account” link
- System shows signup screen
- Player enters a player name
- Player enters a password
    - **Misuse Flow**: Another person sees the entered password and memorizes it
        - **Capture Point**: The password is hidden
        - **Capture Point**: Data is transmitted over an encrypted channel
- Player presses “Create Account” button
- System creates account
    - **Exception Flow**: Name input field is empty or too long
        - System shows an error message
        - Continue with “Player enters a player name”
    - **Exception Flow**: A player with the specified name already exists
        - System show an error message
        - Continue with “Player enters a player name”
    - **Exception Flow**: Password input field is empty or too short
        - System shows an error message
        - Continue with “Player enters a password”
- System shows main menu screen

**Post-conditions**:

- Player has an account
- Player is logged in
- Player is in the main menu

### UC15: Logging in to a Player Account

**ID**: UC14<br>
**Name**: Logging in to a Player Account<br>
**Description**: Allowing the player to log in to their player account<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has an account

**Steps**:

- Player starts the game
- System shows login screen
- Player enters their player name
- Player enters their password
    - **Misuse Flow**: Another person sees the entered password and memorizes it
        - **Capture Point**: The password is hidden
        - **Capture Point**: Data is transmitted over an encrypted channel
- Player presses “Log in” button
- System authenticates the player
    - **Exception Flow**: A player with the specified name does not exist or the password was not correct
        - System shows an error message
        - Continue with “Player enters their player name”
- System shows main menu screen

**Post-conditions**:

- Player is logged in
- Player is in the main menu

### UC16: Changing the Player Name

**ID**: UC15<br>
**Name**: Changing the Player Name<br>
**Description**: Changing the player name<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is logged in
- Player is in the main menu

**Steps**:

- Player presses “Account” button
- System shows account setting screen
- Player presses “Change Player Name” button
- System shows change player name screen
- Player enters a player name
- Player presses “Change Player Name” button
- System updates the name of the player
    - **Exception Flow**: Name input field is empty or too long
        - System shows an error message
        - Continue with “Player enters a player name”
    - **Exception Flow**: A player with the specified name already exists
        - System show an error message
        - Continue with “Player enters a player name”
- System shows main menu screen

**Post-conditions**: n/a

### UC17: Changing the Password

**ID**: UC17<br>
**Name**: Changing the Password<br>
**Description**: Changing the password<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is logged in
- Player is in the main menu

**Steps**:

- Player presses “Account” button
- System shows account setting screen
- Player presses “Change Password” button
- System shows change password screen
- Player enters their old password
- Player enters a new password
    - **Misuse Flow**: Another person sees the entered password and memorizes it
        - **Capture Point**: The password is hidden
        - **Capture Point**: Data is transmitted over an encrypted channel
- Player presses “Change Password” button
- System updates the password of the player
    - **Exception Flow**: Password input field is empty or too short
        - System shows an error message
        - Continue with “Player enters a password”
    - **Exception Flow**: The old password was not correct
        - System shows an error message
        - Continue with “Player enters their old password”
- System shows main menu screen

**Post-conditions**: n/a

### UC18: Deleting the Player Account

**ID**: UC18<br>
**Name**: Deleting the Player Account<br>
**Description**: Deleting the player account<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is logged in
- Player is in the main menu

**Steps**:

- Player presses “Account” button
- System shows account setting screen
- Player presses “Delete this Account” button
- System shows delete account screen
- Player enters their old password
- Player presses “Yes”
- System deletes the account of the player
    - **Exception Flow**: The old password was not correct
        - System shows an error message
        - Continue with “Player enters their old password”
- System shows login screen

**Post-conditions**:

- Player is not logged in

### UC19: Enabling Multiplayer for a World

**ID**: UC19<br>
**Name**: Enabling Multiplayer for a World<br>
**Description**: Opening the world for other players to join until the host player quits the world
or disables multiplayer<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world
- Multiplayer is not enabled for the world

**Steps**:

- Player presses `ESC` on the keyboard
- System shows pause menu
- Player presses “Enable Multiplayer” button
- System shows multiplayer options screen with default configuration (random port, empty blocklist)
- (optional) Player enters a port
- (optional) Player manages access list (includes UC21)
- Player presses “Enable Multiplayer for this World” button
- System enables multiplayer for the world
    - **Exception Flow**: Port input is not a valid port
        - System shows an error message
        - Continue with step “Player enters a port”
    - **Exception Flow**: Port is already in use
        - System shows an error message
        - Continue with step “Player enters a port”
- System shows the game

**Post-conditions**:

- Multiplayer is enabled for the world

### UC20: Changing Multiplayer Options

**ID**: UC20<br>
**Name**: Changing Multiplayer Options<br>
**Description**: Changing options for a world with multiplayer enabled<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player has loaded a world
- Multiplayer is enabled for the world
- Player is the host of the multiplayer world

**Steps**:

- Player presses `ESC` on the keyboard
- System shows pause menu
- Player presses “Multiplayer Options” button
- System shows multiplayer options screen
- (optional) Player manages access list (includes UC21)
- Player presses “Apply & Back to Game” button
- System applies changed options
    - **Exception Flow**: A Player who was denied access has already joined the world
        - System removes player from the world
        - Continue with step “System shows the game”
- System shows the game

**Post-conditions**: n/a

### UC21: Managing Multiplayer Access List

**ID**: UC21<br>
**Name**: Managing Multiplayer Access List<br>
**Description**: Changing the multiplayer to allowlist / blocklist mode and
adding / removing players from the access list<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is in the multiplayer options screen

**Steps**:

- Player presses “Allowlist” or “Blocklist” button
- System changes the multiplayer access mode
- Player enters a player name to add to the list
- Player presses “Allow to join this world” / “Block from joining this world” button
  (depending on the selected access mode)
- System adds the player with the specified name to the list
    - **Exception Flow**: The specified name is the name of the player hosting the multiplayer world
        - System shows an error message
        - Continue with step “Player enters a player name to add to the list”
    - **Exception Flow**: A player with the specified name does not exist
        - System shows an error message
        - Continue with step “Player enters a player name to add to the list”
- Player selects a name from the list
- System highlights the selected player
- Player presses “Remove from list” button
- System removes the player from the list

**Post-conditions**: n/a

### UC22: Joining a Multiplayer World

**ID**: UC22<br>
**Name**: Joining a Multiplayer World<br>
**Description**: Joining a multiplayer world opened by another player<br>
**Area**: Game UI<br>
**Actors**: Player

**Preconditions**:

- Player is in the main menu

**Steps**:

- Player presses “Multiplayer” button
- System shows multiplayer selection screen
- Player enters the address of the multiplayer world
- Player presses “Join Multiplayer” button
- System joins the multiplayer world
    - **Exception Flow**: The address is invalid or there is no world hosted on the address
        - System shows an error message
        - Continue with “Player enters the address of the multiplayer world”
    - **Exception Flow**: The player's access was denied by the player hosting the multiplayer world
        - System show an error message
        - Continue with “Player enters the address of the multiplayer world”
- System shows the game

**Post-conditions**:

- Player has loaded a world
