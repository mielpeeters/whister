use text_io::read;
use whister::{
    fortify::{self, GameSpace, QLearner},
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

    print!("How many iterations? \x1b[1m");
    let iterations: u64 = read!();
    print!("\x1b[0m");
    learner = QLearner::new_with_iter(iterations);

    if from_model != "n" {
        learner.set_q(
            fortify::pickle_to_q(from_model, false)
                .expect("Should give a proper model (relative) path that exists"),
        );
    }

    // extend training based on an existing model or start over
    print!("Self play against model? [modelname/n]: \x1b[1m");
    let opponent: String = read!();
    print!("\x1b[0m");

    learner.train(&mut Game::new(), opponent);

    let mut q = learner.get_q();

    loop {
        for _ in 0..40001 {
            let mut best_action = fortify::best_action_score(&mut q, &game.state(), 0.0).0;
            let alowed = game.actions();

            if !alowed.iter().any(|a| *a == best_action) {
                best_action = alowed[0];
            }

            let best_card_id = game.action_card_id(&best_action);

            game.agent_plays_round(best_card_id, &mut None);
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

    fortify::q_to_pickle(&q, answer, false);
}
