use whister::{fortify::QLearner, game::Game, gamestate::GameState};

fn main() {
    let mut learner: QLearner<GameState>;
    let mut discounts = Vec::new();

    for i in 1..10 {
        discounts.push(i as f64 / 10.0);
    }

    let mut lrs = Vec::new();
    for i in 1..10 {
        lrs.push(i as f64 / 50.0);
    }

    let mut wtr = csv::Writer::from_path("out.csv").unwrap();
    wtr.write_record(&["discount", "learning_rate", "score"])
        .unwrap();

    for discount in &discounts {
        for lr in &lrs {
            let mut game: Game = Game::new();

            learner = QLearner::new_with_iter(1000000);

            learner.enable_self_play();
            learner.rate = *lr;
            learner.discount = *discount;

            learner.train(&mut Game::new());

            let q = learner.get_q();

            for _ in 0..40001 {
                let best_card_id = game.best_card_id_ai(&q);

                // play rounds against a rule based opponent
                game.agent_plays_round(best_card_id, &None);
            }

            let score = game.get_ai_score() as f32 / (10000.0);

            wtr.write_record(&[
                format!("{}", discount).as_str(),
                format!("{}", lr).as_str(),
                format!("{}", score).as_str(),
            ])
            .unwrap();
        }
    }

    wtr.flush().unwrap();
}
