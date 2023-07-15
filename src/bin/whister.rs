/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/
use whister::{
    fortify,
    game::Game, show,
};

fn main() {
    show::clear();
    let mut game = Game::new();
    game.add_human_players(1).unwrap();

    let mut count = 0;

    let q = fortify::data::select_model(false);

    if q.is_none() {
        println!("A rule based opponent will be used!");
        show::wait();
        show::wait();
    }

    print!("\nNeed instructions? [Y/n]: \x1b[1m");
    let answer = show::yes_or_no(true);
    print!("\x1b[0m");
    if answer {
        Game::instructions();
    }

    loop {
        game.play_deal(&q);

        game.new_round();
        game.show_scores();

        print!("Play another round? [Y/n]: \x1b[1m");
        let answer = show::yes_or_no(true);
        print!("\x1b[0m");
        if !answer {
            break;
        }

        count += 1;
        if count > 30000 {
            break;
        }
    }
}
