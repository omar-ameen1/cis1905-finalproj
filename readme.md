# Super Cool 2D PVP Shooter Game

## Description
You can shoot, you can play with friends, you can place walls to defend yourself or hide! :D

## Setup:
1. Make sure you have Rust installed.
2. Make sure you have matchbox_server installed:
    ```bash
    cargo install matchbox_server
    ```
3. Clone this repository.
4. Run the server:
    ```bash
    matchbox_server
    ```
5. Make sure that the output of the server is:
    ```
   INFO Matchbox Signaling Server: 0.0.0.0:3536
   ```
6. In [network_manager.rs](src/network_manager.rs), set NUM_PLAYERS to the number of players you want to play with.
7. Run the game:
    ```bash
    cargo run
    ```
8. Note: The game will not work until NUM_PLAYERS clients have connected. To test it yourself locally, run the game from multiple terminals.