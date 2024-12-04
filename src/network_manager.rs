use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use bevy_ggrs::*;
use crate::AppState;
use crate::GameConfig;

/// Resource for storing the game's random seed
#[derive(Resource, Default, Clone, Copy, Debug, Deref, DerefMut)]
pub struct RandomSeed(u64);

pub (crate) const NUM_PLAYERS: usize = 2;

/// Registers the networking systems to the app
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Connecting), initialize_socket)
        .add_systems(
            Update,
            wait_for_players.run_if(in_state(AppState::Connecting)),
        );
}

/// Initializes the network socket for matchmaking
fn initialize_socket(mut commands: Commands) {
    let matchbox_url =
        String::from("ws://0.0.0.0:3536/cis1905?next=$") + &NUM_PLAYERS.to_string();
    info!("Connecting to {}", matchbox_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(matchbox_url));
}

/// Waits for all players to connect before starting the game
fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // If the channel isn't ready yet, just return
    if socket.get_channel(0).is_err() {
        return;
    }

    socket.update_peers();
    let connected_players = socket.players();

    let required_players = NUM_PLAYERS;
    if connected_players.len() < required_players {
        info!(
            "Waiting for {} more player(s)...",
            required_players - connected_players.len()
        );
        return;
    }

    info!("All players have connected!");

    // Generate a random seed based on connected peer IDs
    let own_id = socket.id().expect("Failed to get socket ID").0.as_u64_pair();
    let mut seed = own_id.0 ^ own_id.1;
    for peer in socket.connected_peers() {
        let peer_id = peer.0.as_u64_pair();
        seed ^= peer_id.0 ^ peer_id.1;
    }

    commands.insert_resource(RandomSeed(seed));

    let mut session_builder = ggrs::SessionBuilder::<GameConfig>::new()
        .with_num_players(required_players)
        .with_input_delay(2);

    for (index, player) in connected_players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, index)
            .expect("Failed to add player to session");
    }

    let communication_channel = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder
        .start_p2p_session(communication_channel)
        .expect("Failed to start P2P session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
    next_state.set(AppState::InGame);
}
