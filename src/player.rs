use crate::{
    deck::Deck,
    card::Card,
};

/// One of the four players in the colour whist game. 
pub struct Player {
    pub cards: Deck,
}

impl Player {
    pub fn new_take_cards(deck: &mut Deck, amount: usize) -> Player {
        let pulled = deck.pull_cards(amount);
        Player { cards: pulled}
    }

    /// show the cards this player is holding, by first sorting them!
    pub fn show_cards(&mut self) {
        self.cards.show_sort();
    }

    pub fn random_card(&self) -> Card {
        self.cards.peek()
    }

    pub fn has_cards(&self) -> bool {
        self.cards.size() > 0
    }
}

