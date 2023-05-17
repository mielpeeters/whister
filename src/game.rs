/*!
 * This module defines a game of colour whist, which consists of tricks and the current table
 */

use crate::{
    playingcards::{Deck, Card},
    player::Player,
};

pub struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    tricks: Vec<Deck>,
    /// table keeps track of which cards are 
    /// on the table already
    table: Deck,
    pub players: [Player; 4]
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        let mut deck = Deck::new_full();
        deck.shuffle();
        let tricks: Vec<Deck> = Vec::new();
        let table: Deck = Deck::new_empty();
        
        // create new players
        let player_one = Player::new_take_cards(&mut deck, 13);
        let player_two = Player::new_take_cards(&mut deck, 13);
        let player_three = Player::new_take_cards(&mut deck, 13);
        let player_four = Player::new_take_cards(&mut deck, 13);
        // deck is empty now

        let players = [player_one, player_two, player_three, player_four];

        Game{ tricks, table, players }
    }

    pub fn trick(&mut self) -> Result<(), String> {
        if self.table.size() == 4 {
            let new_trick = self.table.pull_cards(4);
            self.tricks.push(new_trick);
            Ok(())
        } else {
            Err(String::from("There are not exactly four cards on the table."))
        }
    }

    fn play(&mut self, card: Card) -> Result<(), String> {
        if self.table.size() < 4 {
            self.table.add(card);
            Ok(())
        } else {
            Err(String::from("There are already four cards on the table. Can't play any more."))
        }
    }

    pub fn show_table(&self) {
        self.table.show();
    }

    pub fn play_card(&mut self, player: usize, card: usize) {
        let played = self.players[player].cards.remove(card);
        self.play(played).expect("Couldn't play card");
    }
}