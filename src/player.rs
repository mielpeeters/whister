/*!
 * Player of the colour whist game.
 */

use crate::{
    card::Card,
    deck::{CardID, Deck},
    suit::Suit,
};

/// One of the four players in the colour whist game.
#[derive(Hash, PartialEq, Eq)]
pub struct Player {
    pub cards: Deck,
}

impl Player {
    /// Create a new player, pulling `amount` cards from `deck`
    pub fn new_take_cards(deck: &mut Deck, amount: usize) -> Player {
        let pulled = deck.pull_cards(amount);
        Player { cards: pulled }
    }

    /// show the cards this player is holding, by first sorting them!
    pub fn show_cards(&mut self) {
        self.cards.show_sort();
    }

    /// look at a random card of this player's deck
    pub fn random_card(&self) -> &Card {
        self.cards.peek()
    }

    /// Does this player have any cards left?
    pub fn has_cards(&self) -> bool {
        self.cards.size() > 0
    }

    /// Does this player have any cards of this suit?
    pub fn can_follow(&self, suit: Suit) -> bool {
        self.cards.has_suit(&suit)
    }

    pub fn card(&self, card: CardID) -> &Card {
        self.cards.card(card)
    }

    pub fn selected_card(&self) -> &Card {
        self.cards.selected()
    }

    pub fn selected_id(&self) -> CardID {
        self.cards.selected_id()
    }
}
