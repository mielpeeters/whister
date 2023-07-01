use whister::{fortify::QLearner, game::Game};


fn main() {
    let game = Game::new();
    let mut learner = QLearner::new(game);

    learner.train();
}