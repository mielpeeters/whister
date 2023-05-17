/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/

use whister::{
    playingcards::Deck, 
    game::Game
};
use rand::{self, Rng};

fn main() {
    let mut deck = Deck::new_full();
    let mut game = Game::new();

    // random shuffle
    deck.shuffle(); 

    for i in 0..4 {
        println!("Player {i}'s deck:");
        game.players[i].show_cards();
    }

    let mut rng = rand::thread_rng();

    for _ in 0..13 {

        for i in 0..4 {
            let random_index = rng.gen_range(0..game.players[i].cards.size());
            game.play_card(i, random_index);
        }
        game.players[0].show_cards();
        game.show_table();
        game.trick().expect("Couldn't finish trick");
    }
            

            
}
