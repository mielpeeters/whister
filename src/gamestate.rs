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

impl State for BidState {
    type A = Bid;
}

impl Bid {
    /// match the Bid to get a value which is used for ordering bids
    fn value(&self) -> i8 {
        match self {
            Bid::Accept(_, 8) => 0,
            Bid::Solo(_, 5) => 1,
            Bid::Accept(_, 9) => 2,
            Bid::Solo(_, 6) => 3,
            Bid::Accept(_, 10) => 4,
            Bid::Solo(_, 7) => 5,
            Bid::Accept(_, 11) => 6,
            Bid::SmallMisery => 7,
            Bid::Accept(_, 12) => 8,
            Bid::Solo(_, 8) => 9,
            Bid::Accept(_, 13) => 10,
            Bid::Abondance(_, 9) => 11,
            Bid::Trull => 12,
            Bid::LargeMisery => 13,
            Bid::Abondance(_, 10) => 14,
            Bid::Abondance(_, 11) => 15,
            Bid::OpenMisery => 16,
            Bid::Abondance(_, 12) => 17,
            Bid::SoloSlim(_) => 18,
            Bid::Pass => -1,
            Bid::Wait => -1,
            Bid::Propose(_) => -1,
            Bid::PassParole => -1,
            _ => -1,
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let value_comp = self.value().cmp(&other.value());
        if value_comp == std::cmp::Ordering::Equal {
            if let Bid::Accept(s1, _) = self {
                if let Bid::Accept(s2, _) = other {
                    Some(s1.cmp(s2))
                } else {
                    None
                }
            } else if let Bid::Solo(s1, _) = self {
                if let Bid::Solo(s2, _) = other {
                    Some(s1.cmp(s2))
                } else {
                    None
                }
            } else if let Bid::Abondance(s1, _) = self {
                if let Bid::Abondance(s2, _) = other {
                    Some(s1.cmp(s2))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            Some(value_comp)
        }
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
