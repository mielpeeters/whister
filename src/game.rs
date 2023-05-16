use crate::playingcards::{Deck, Card};

pub struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    pub tricks: Vec<Deck>,
    /// table keeps track of which cards are 
    /// on the table already
    pub table: Vec<Card>,
}

impl Game {
    pub fn new() -> Game {
        let tricks: Vec<Deck> = Vec::new();
        let table: Vec<Card> = Vec::new();
        Game{ tricks, table }
    }
}