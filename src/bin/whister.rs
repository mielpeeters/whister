/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/
use text_io::read;
use whister::{
    fortify,
    game::Game, show,
};

fn main() {
    let mut game = Game::new();
    game.add_human_players(1).unwrap();

    let mut count = 0;

    print!("Insert AI model name (random for rulebased opponent): \x1b[1m");
    let model_name: String = read!();
    print!("\x1b[0m");

    let q =  fortify::pickle_to_q(model_name.as_str(), false);

    if q.is_none() {
        println!("A rule based opponent will be used!");
        show::wait();
        show::wait();
    }

    print!("\nNeed instructions? [y/n]: \x1b[1m");
    let instr: String = read!();
    print!("\x1b[0m");
    if instr.contains('y') {
        Game::instructions();
    }

    loop {
        for _ in 0..13 {
            game.play_deal(&q);
        }

        game.new_deal();
        game.show_scores();

        println!("Play another deal? (false / true)");
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
