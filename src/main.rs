/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/

use whister::{
    deck::Deck, 
    game::Game
};
use text_io::read;

fn main() {
    let mut deck = Deck::new_full();
    let mut game = Game::new();

    // random shuffle
    deck.shuffle(); 

    let mut count = 0;

    loop {
        for _ in 0..13 {
            game.play_round();
        }

        game.show_scores();

        println!("Play another round? (false / true)");
        let answer: bool = read!();
        if !answer {
            break;
        }
        
        count += 1;
        if count > 30000 {
            break;
        }
        
        game.new_round();
    }

}
