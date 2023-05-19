/*!
 * This module defines a game of colour whist, which consists of tricks and the current table
 */

use crate::{
    deck::Deck,
    card::Card,
    player::Player, colour::Colour,
};
use rand::{thread_rng, Rng};
use text_io::read;

pub struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    tricks: Vec<Deck>,
    /// table keeps track of which cards are 
    /// on the table already
    table: Deck,
    pub players: [Player; 4],
    turn: usize,
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

        Game{ tricks, table, players, turn: 0 }
    }

    fn show_last_trick(&self) {
        if self.tricks.len() > 0 {
            println!("Played trick:\n{}",self.tricks[self.tricks.len()-1]);
        } else {
            let deck = Deck::new_empty();
            println!("Played trick:\n{}",deck);
        }
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

    fn winner(&self) -> usize {
        let mut best_card = self.table.card(0);
        let mut winner = self.turn;

        for i in 1..4 {
            let new_card = self.table.card(i);

            if new_card.colour == best_card.colour {
                // new card could be the better one
                if new_card > best_card {
                    best_card = new_card;
                    winner = (self.turn + i) % 4;
                }

            } // TODO: add trouve functionality (should come before current if! always wins!)
        }

        winner
    }

    /// returns a vector of alowed cards for this player, in this round
    fn alowed_cards(&self, player: usize) -> Vec<usize> {
        // if the table is empty, every card is alowed
        let mut result: Vec<usize> = Vec::new();

        let player = &self.players[player];

        if self.table.size() != 0 { // i'm possibly restricted to the first-layed card this trick
            // TODO: implement alowable card rules
            //  - check first card type
            //  - check if I have any
            //      - if not, `if` scenario should happen -> turn these around!
            let first_colour = self.table.card(0).colour;

            if player.cards.has_colour(&first_colour) {
                // only return indexes of cards of said colour
                for i in 0..player.cards.size() {
                    if player.cards.colour_at(i) == first_colour {
                        result.push(i);
                    }
                }
                return result;
            }
        }

        // if I got here, i am free to play any card
        for i in 0..player.cards.size() {
            result.push(i);
        }

        result    
    }

    pub fn play_round(&mut self) {
        let mut rng = thread_rng();
        
        for i in self.turn..self.turn+4 {
            let player = i % 4;

            if player == 0 {
                println!("\x1b[1J\x1b[H");
                self.show_last_trick();
                println!("Current table: \n{}\n", self.table);
                println!("Your hand:");
                self.players[0].show_cards();
                print!("Enter a suit (S,C,D,H):\n");
                let colour: String = read!();

                let colour = match colour.as_str() {
                    "H" | "h" => Colour::Hearts,
                    "D" | "d" => Colour::Diamonds,
                    "C" | "c" => Colour::Clubs,
                    "S" | "s" => Colour::Spades,
                    &_ => Colour::Hearts,
                };

                print!("Enter a value (1-13):\n");
                let number: u8 = read!();

                let card = Card{colour, number};

                let idx = self.players[0].cards.index_of(&card);

                self.play_card(0, idx);
            } else {
                // get alowed indeces
                let playable = self.alowed_cards(player);
    
                // get a random index from these
                // TODO: here lies the AI part of the project!
                let rand_index = rng.gen_range(0..playable.len());
    
                // play that card
                self.play_card(player, playable[rand_index]);
            }
        }

        // TODO remove this line, change to winner of this trick (using the trick function's output maybe?)
        self.turn = self.winner();

        self.trick().expect("Couldn't play trick in play_round");
    }
}