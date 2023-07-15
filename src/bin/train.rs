use text_io::read;
use whister::{
    fortify::{self, QLearner},
    game::Game,
    gamestate::GameState, show,
};

fn main() {
    let mut learner: QLearner<GameState>;
    let mut game: Game = Game::new();

    let q = fortify::data::select_model(true);

    print!("How many iterations? \x1b[1m");
    let iterations: u64 = read!();
    print!("\x1b[0m");
    learner = QLearner::new_with_iter(iterations);

    if let Some(q) = q {
        learner.set_q(q);
    }

    // extend training based on an existing model or start over
    print!("Enable self-play? [Y/n]: \x1b[1m");
    let answer = show::yes_or_no(true);
    print!("\x1b[0m");
    if answer {
        learner.enable_self_play();
        println!("-> self play is enabled");
    }

    learner.train(&mut Game::new());

    let q = learner.get_q();

    loop {
        for _ in 0..40001 {
            let best_card_id = game.best_card_id_ai(&q);

            // play rounds against a rule based opponent
            game.agent_plays_round(best_card_id, &None);
        }

        game.show_scores();

        print!("Play another round? [y/N]: \x1b[1m");
        let answer = show::yes_or_no(false);
        print!("\x1b[0m");
        if !answer {
            break;
        }
    }

    print!("Save this model? [modelname/N]: \x1b[1m");
    let answer = show::get_answer();
    print!("\x1b[0m");
    if let Some(answer) = answer {
        if answer == "N" || answer == "n" || answer.is_empty() {
            return
        }
        println!("Saving model as \x1b[3m{:?}\x1b[0m", answer);
        fortify::data::q_to_pickle(&q, answer, false).expect("Should be able to save");
    } 

}
