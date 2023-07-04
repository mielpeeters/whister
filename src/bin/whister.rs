/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/
use whister::{
    game::Game, fortify::{QLearner, GameState}, card::Card
};
use text_io::read;

fn main() {
    let mut game = Game::new();

    println!("Will an AI player participate? [y/n]");
    let ai_player: String = read!();

    let mut count = 0;

    if ai_player == "y" {
        let mut learner: QLearner;

        println!("From model? [modelname/n]");
        let from_model: String = read!();
        if from_model != "n" {
            learner = QLearner::new(Game::new());
            learner.import_from_model(from_model.clone(), true);
        } else {
            println!("how many iterations?");
            let iterations: u64 = read!();
    
            learner = QLearner::new_with_iter(Game::new(), iterations);
    
            learner.train();

        }


        let mut state: GameState = GameState { ..Default::default() };
        let mut played_card: Card = Card { ..Default::default() };
        
        loop {
            println!("Show AI's internal logic? [y/n]");
            let slow: String = read!();
            let slow = matches!(slow.as_str(), "y");

            let gamesize = match slow {
                true => 13,
                false => 40001,
            };

            for _ in 0..gamesize {
                let mut best_action = *learner.best_action_score(&game.state()).0;
                let alowed = learner.alowed_actions(&game);

                if !alowed.iter().any(|a| *a == best_action) {
                    best_action = alowed[0];
                }

                let best_card_id = learner.action_card_id(&best_action, &game);
                
                if slow {
                    played_card = game.players.get(0).unwrap().card(best_card_id).clone();
                    state = game.state();
                    game.agent_plays_round_slowly(best_card_id);
                } else {
                    game.agent_plays_round(best_card_id);
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

        if from_model != "n" {
            return
        }

        println!("Save this model? [modelname/n]");
        let answer: String = read!();
        if answer == "n" {
            return
        }
        learner.save_result(answer, true);
        
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
