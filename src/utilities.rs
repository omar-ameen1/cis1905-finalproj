use bevy::prelude::*;
use bevy::reflect::List;
use crate::GamePhase;
use crate::network_manager::NUM_PLAYERS;

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RoundTimer(Timer);

#[derive(Resource, Default, Clone)]
// Tuple with capacity NUM_PLAYERS
pub struct PlayerScores {
    scores: Vec<u64>,
}

impl PlayerScores {
    pub fn new() -> Self {
        Self {
            scores: vec![0; NUM_PLAYERS],
        }
    }

    pub fn get(&self, player: usize) -> u64 {
        self.scores[player]
    }

    pub fn set(&mut self, player: usize, score: u64) {
        self.scores[player] = score;
    }
}

impl Default for RoundTimer {
    fn default() -> Self {
        RoundTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}
pub fn round_over_timer(
    mut timer: ResMut<RoundTimer>,
    mut state: ResMut<NextState<GamePhase>>,
    time: Res<Time>,
) {
    println!("round_end_timeout");
    timer.tick(time.delta());

    if timer.just_finished() {
        state.set(GamePhase::ActiveRound);
    }
}
