/*!
 * This module implements the gamestate trait to be used in reinforcement learning.
 */

use serde::{Deserialize, Serialize};

use crate::{fortify::State, suit::Suit};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameState {
    /// can this player follow the current table?
    pub can_follow: bool,
    /// does this player have a winning card compared to current table?
    pub have_higher: bool,
    /// what was the first played suit?
    pub first_suit: i8,
    /// of which suits does this player have the highest card?
    pub has_highest: [bool; 4],
    /// does this player have any trumps left?
    pub have_trump: bool,
    /// how many cards does this player have left?
    pub nb_cards: u8,
    /// how many players are out of which suits?
    pub nb_out_of: u8,
}

/// All possible actions that the agent can take.
/// Technically, the action could be described as just a Card, but
/// these are realistic moves for many scenarios, to reduce the state-action space.
#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize, Copy, Clone)]
pub enum Action {
    /// play the lowest card you can
    PlayWorst(Suit),
    /// play a higher card, but the lowest you can
    RaiseLow,
    /// play a higher card, the highest you can
    RaiseHigh,
    /// play the highest trump card you can
    TrumpHigh,
    /// play the lowest trump card that buys it
    TrumpLow,
    /// play a card you know is the best one
    PlayBest(Suit),
    /// come with a card you know is the best one
    ComeBest,
}

impl Default for Action {
    fn default() -> Self {
        Self::PlayWorst(Suit::Clubs)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BidState {}

/// The types of bidding a player can choose from
#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub enum Bid {
    #[default]
    // don't join or start a contract
    Pass,
    // only player left of dealer, wait and decide to accept or not
    Wait,
    // start a contract in a new suit
    // 8 or more, with a partner
    Propose(Suit),
    // accept an open contract
    Accept(Suit, u8),
    // go solo, only if no others left to accept or when others passed
    Solo(Suit, u8),
    // get no tricks, put away one card
    SmallMisery,
    // get no tricks with all cards
    LargeMisery,
    // get no tricks with open cards
    OpenMisery,
    // 9 or more tricks solo
    // only as a first bid
    Abondance(Suit, u8),
    // 13 tricks solo, for real mad lads
    // only as a first bid
    SoloSlim(Suit),
    // accepted and at least 11 tricks bid, proposer continues to bid
    PassParole,
    // three or four aces dealt to one player
    Trull,
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
        match self.value().cmp(&other.value()) {
            // If the bids are equal in score, they are separated in suit!
            std::cmp::Ordering::Equal => match (self, other) {
                (Bid::Accept(s1, _), Bid::Accept(s2, _)) => Some(s1.cmp(s2)),
                (Bid::Solo(s1, _), Bid::Solo(s2, _)) => Some(s1.cmp(s2)),
                (Bid::Abondance(s1, _), Bid::Abondance(s2, _)) => Some(s1.cmp(s2)),
                _ => None,
            },
            // If bids are not equal in score, just return their ordering
            other => Some(other),
        }
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
