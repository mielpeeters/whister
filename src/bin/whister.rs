/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/
use whister::{
    game::Game, fortify::QLearner
};
use text_io::read;

fn main() {
    let mut game = Game::new();
    game.add_human_players(1).unwrap();
    
    let mut count = 0;

    print!("\nInsert AI model name: \x1b[1m");
    let model_name: String = read!();
    print!("\x1b[0m");


    let mut learner = QLearner::new(Game::new());
    learner.import_from_model(model_name, false);

    loop {
        for _ in 0..13 {
            game.play_round(&mut learner);
        }

        game.new_round();
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
        
    }
}
