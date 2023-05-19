/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/

use whister::{
    deck::Deck, 
    game::Game
};

fn main() {
    let mut deck = Deck::new_full();
    let mut game = Game::new();

    // random shuffle
    deck.shuffle(); 

    for _ in 0..13 {
        game.play_round();
    }

    game.finish();
}
