use text_io::read;
use whister::{
    fortify::{QLearner, GameSpace},
    game::Game,
    gamestate::GameState,
};

fn main() {
    let mut learner: QLearner<GameState>;
    let mut game: Game = Game::new();

    // extend training based on an existing model or start over
    print!("Start from existing model? [modelname/n]: \x1b[1m");
    let from_model: String = read!();
    print!("\x1b[0m");

    println!("how many iterations?");
    let iterations: u64 = read!();
    learner = QLearner::new_with_iter(iterations);

    if from_model != "n" {
        learner.import_from_model(from_model, false);
    }

    learner.train(&mut Game::new());

    loop {
        for _ in 0..40001 {
            let mut best_action = *learner.best_action_score(&game.state()).0;
            let alowed = game.actions();

            if !alowed.iter().any(|a| *a == best_action) {
                best_action = alowed[0];
            }

            let best_card_id = game.action_card_id(&best_action);

            game.agent_plays_round(best_card_id);
        }

        game.show_scores();

        println!("Play another round? [y/n]");
        let answer: String = read!();
        let answer: bool = matches!(answer.as_str(), "y");
        if !answer {
            break;
        }
    }

    println!("Save this model? [modelname/n]");
    let answer: String = read!();
    if answer == "n" {
        return;
    }

    learner.save_result(answer, false);
}
