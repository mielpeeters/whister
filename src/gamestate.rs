/*!
 * This module implements the gamestate trait to be used in reinforcement learning.
 */

use serde::{Deserialize, Serialize};

use crate::fortify::State;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameState {
    pub can_follow: bool,
    pub have_higher: bool,
    pub first_suit: i8,
    pub has_highest: [bool; 4],
    pub have_trump: bool,
    pub nb_cards: usize
}

/// All possible actions that the agent can take.
/// Technically, the action could be described as just a Card, but
/// these are realistic moves for many scenarios, to reduce the state-action space.
#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize, Copy, Clone, Default)]
pub enum Action {
    #[default]
    /// play the lowest card you can
    PlayWorst,
    /// play a higher card, but the lowest you can
    RaiseLow,
    /// play a higher card, the highest you can
    RaiseHigh,
    /// play the highest trump card you can
    TrumpHigh,
    /// play the lowest trump card that buys it
    TrumpLow,
    /// play a card you know is the best one
    PlayBest,
    /// come out with a card you know is the best
    ComeBest,
}

impl State for GameState {
    type A = Action;
}