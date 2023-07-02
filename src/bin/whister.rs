/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/
use whister::{
    game::Game, fortify::QLearner
};
use text_io::read;

fn main() {
    let mut game = Game::new();

    println!("Will an AI player participate? (true / false)");
    let ai_player: bool = read!();

    let mut count = 0;

    if ai_player {
        println!("how many iterations?");
        let iterations: u64 = read!();

        let mut learner = QLearner {
            game: Game::new(),
            iterations,
            ..Default::default()
        };

        learner.train();

        loop {
            for _ in 0..40000 {
                let mut best_action = *learner.best_action_score(&game.state()).0;
                let alowed = learner.alowed_actions(&game);

                if !alowed.iter().any(|a| *a == best_action) {
                    best_action = alowed[0];
                }

                let best_card_id = learner.action_card_id(&best_action, &game);

                let played_card = game.players.get(0).unwrap().card(best_card_id).clone();
                let state = game.state();

                game.agent_plays_round_slowly(best_card_id);

                println!("Played Card: {}\n", played_card);
                println!("Played CardID: {}\n", best_card_id);
                println!("Played Action: {}\n", best_action);

                println!("From state: {}\n", state);

                println!("Press [enter] to continue");
                let _: String = read!();
            }

            game.show_scores();
            
            println!("Play another round? (false / true)");
            let answer: bool = read!();
            if !answer {
                break;
            }
        }

        return 
    } 

    println!("How many human players will participate?");
    let answer: usize = read!();
    game.add_human_players(answer).expect("Unable to add human players.");


    loop {
        for _ in 0..13 {
            game.play_round();
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
