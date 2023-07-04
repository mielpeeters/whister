use text_io::read;
use whister::{
    card::Card,
    fortify::{GameState, QLearner},
    game::Game,
};

fn main() {
    let mut learner: QLearner;
    let mut game: Game = Game::new();

    // extend training based on an existing model or start over
    print!("Start from existing model? [modelname/n]: \x1b[1m");
    let from_model: String = read!();
    print!("\x1b[0m");

    println!("how many iterations?");
    let iterations: u64 = read!();
    learner = QLearner::new_with_iter(Game::new(), iterations);

    if from_model != "n" {
        learner.import_from_model(from_model, false);
    }

    print!("Type opponent? [modelname/n]: \x1b[1m");
    let opponent: String = read!();
    print!("\x1b[0m");

    if opponent != "n" {
        learner.set_opponent(opponent.clone());
    }

    learner.train();

    let mut state: GameState = GameState {
        ..Default::default()
    };
    let mut played_card: Card = Card {
        ..Default::default()
    };

    loop {
        println!("Show AI's internal logic? [y/n]");
        let slow: String = read!();
        let slow = matches!(slow.as_str(), "y");

        let gamesize = match slow {
            true => 13,
            false => 40001,
        };

        let mut opponent = {
            let mut opp = QLearner::new(Game::new());
            opp.import_from_model(opponent.clone(), false);
            opp
        };

        for _ in 0..gamesize {
            let mut best_action = *learner.best_action_score(&game.state(0)).0;
            let alowed = learner.alowed_actions(&game, 0);

            if !alowed.iter().any(|a| *a == best_action) {
                best_action = alowed[0];
            }

            let best_card_id = learner.action_card_id(&best_action, &game, 0);

            if slow {
                played_card = game.players.get(0).unwrap().card(best_card_id).clone();
                state = game.state(0);
                game.agent_plays_round_slowly(best_card_id, &mut opponent);
            } else {
                game.agent_plays_round(best_card_id, &mut opponent);
            }

            if slow {
                println!("Played Card: {}\n", played_card);
                println!("Played CardID: {}\n", best_card_id);
                println!("Played Action: {}\n", best_action);
                println!("Alowed Actions: {:?}\n", alowed);

                println!("From state: {}\n", state);

                println!("Press [enter] to continue");
                let _: String = read!();
            }
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
