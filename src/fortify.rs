/*!
 * This module implements approximate Q-learning for the whister card game.
 *
 * Multiple difficulty levels are defined, by giving the agent increasing amounts of data to
 * train on. Trained models will be supplied when they are finished (basically a serialized Q hash map).
 */
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::{deck::CardID, suit::Suit};
use crate::game::Game;
use itertools::Itertools;
use rand::{Rng};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, fmt::Display};

/// All possible actions that the agent can take.
/// Technically, the action could be described as just a Card, but
/// these are realistic moves for many scenarios, to reduce the state-action space.
#[derive(Hash, PartialEq, Eq, Debug, PartialOrd, Ord, Serialize, Deserialize, Copy, Clone)]
pub enum Action {
    /// play the lowest card you can
    PlayWorst,
    /// play a higher card, but the lowest you can
    RaiseLow,
    /// play a higher card, the highest you can
    RaiseHigh,
    /// play the highest trump card you can
    TrumpHigh,
    /// play the lowest trump card that buys it
    TrumpLow,
    /// play a random card
    Random,
    /// play a card you know is the best one
    PlayBest,
    /// come out with a card you know is the best
    ComeBest,
}

const GROUP_SIZE: usize = 3;
const EXPLORE: u8 = 30;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameState {
    pub can_follow: bool,
    pub nb_trump: usize,
    pub first: bool,
    pub first_suit: i8,
    pub has_highest: [bool; 4]
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            can_follow: true,
            nb_trump: 13,
            first: true,
            first_suit: 0,
            has_highest: [false; 4]
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new()
    }
}

pub type Q = HashMap<GameState, HashMap<Action, f64>>;

pub struct QLearner {
    pub q: Q,
    pub rate: f64,
    pub discount: f64,
    pub initial_value: f64,
    pub iterations: u64,
    pub game: Game,
}

impl QLearner {
    pub fn new(game: Game) -> Self {
        let q = HashMap::new();
        QLearner {
            q,
            rate: 0.1,
            discount: 0.8,
            initial_value: 0.0,
            iterations: 100000,
            game,
        }
    }

    pub fn train(&mut self) {
        let mut count = 0;
        loop {
            let current_state = self.game.state();
            let current_values = self.q.get(&current_state).unwrap_or(&HashMap::new());

            // determine a new action to take, from current state
            let action = self.new_action(&current_state);

            self.take_action(&action);

            // reward is the reward that's coupled with this action
            let reward = self.game.reward();
            let best_future = *self.best_action_score(&self.game.state()).1;

            // new value to assign to Q(s,a)
            let v: f64 = {
                // get the old value of Q(s,a) if it is available
                let old_value = self
                    .q
                    .get(&current_state)
                    .and_then(|m| m.get(&action))
                    .unwrap_or(&self.initial_value);

                *old_value + self.rate * (reward + self.discount * best_future - *old_value)
            };

            self.q
                .entry(current_state)
                .or_insert_with(HashMap::new)
                .insert(action, v);

            count += 1;

            if count > self.iterations {
                break;
            }
            print!("\x1b[2K\x1b[0G iteration {} / {}", count, self.iterations);
        }

        println!("\nThere are {:?} states that have been explored...", self.q.len());
        
        // self.show_result();
        // self.save_result();
    }

    /// determine the best action in current state, based on the q function
    pub fn best_action_score(&mut self, state: &GameState) -> (&Action, &f64) {
        let best = self
            .q
            .entry(*state)
            .or_insert_with(HashMap::new)
            .iter()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(Ordering::Equal));

        best.unwrap_or((&Action::RaiseHigh, &0.0))
    }

    /// determine the action the agent takes while exploring the statespace
    fn new_action(&mut self, state: &GameState) -> Action {
        let alowed = self.alowed_actions(&self.game);

        let exploit_factor = rand::thread_rng().gen_range(0..100);
        
        let best = self.best_action_score(state);

        if alowed.contains(best.0) && exploit_factor > EXPLORE {
            return *best.0
        }

        let num = rand::thread_rng().gen_range(0..alowed.len());

        *alowed.get(num).unwrap()
    }

    pub fn alowed_actions(&self, game: &Game) -> Vec<Action> {
        let playable = game.alowed_cards(0);
        let better = game.better_cards_of(0, &playable);

        let mut alowed: Vec<Action> = Vec::new();
        alowed.push(Action::PlayWorst);
        alowed.push(Action::PlayBest);
        // alowed.push(Action::Random);

        let state = game.state();
        if state.first && state.has_highest.iter().any(|highest| *highest) {
            alowed.push(Action::ComeBest);
        }

        if game.players[0].can_follow(Suit::Hearts) && (game.first() || !game.can_follow(0)) {
            alowed.push(Action::TrumpHigh);
            alowed.push(Action::TrumpLow);
        }

        if game.can_follow(0) && !better.is_empty() && !state.first {
            alowed.push(Action::RaiseLow);
            alowed.push(Action::RaiseHigh);
        }

        alowed
    }

    fn take_action(&mut self, action: &Action) {
        let card_id = self.action_card_id(action, &self.game);
        self.game.agent_plays_round(card_id);
    }

    pub fn action_card_id(&self, action: &Action, game: &Game) -> CardID {
        let playable = game.alowed_cards(0);

        match action {
            Action::PlayWorst => {
                game.lowest_card_of(0, &playable)
            }
            Action::RaiseLow => {
                let better = game.better_cards_of(0, &playable);
                game.lowest_card_of(0, &better)
            }
            Action::RaiseHigh => {
                let better = game.better_cards_of(0, &playable);
                game.highest_card_of(0, &better)
            }
            Action::TrumpHigh => {
                let trumps = game.of_which_suit(0, &playable, 3);
                game.highest_card_of(0, &trumps)
            }
            Action::TrumpLow => {
                let trumps = game.of_which_suit(0, &playable, 3);
                game.lowest_card_of(0, &trumps)
            }
            Action::Random => {
                let rnd = rand::thread_rng().gen_range(0..playable.len());
                playable[rnd]
            }
            Action::PlayBest => {
                game.highest_card_of(0, &playable)
            },
            Action::ComeBest => {
                let state = game.state();
                let suit = state.has_highest.iter().position_max().unwrap();
                let suit_ids = game.of_which_suit(0, &playable, suit);

                game.highest_card_of(0, &suit_ids)
            },
        }
    }

    fn show_result(&self) {
        for result in self.q.keys().sorted() {
            println!("State:");
            println!("{}", result);

            for action in self.q.get(result).unwrap().keys().sorted() {
                println!(
                    "  {}: {}",
                    action,
                    self.q.get(result).unwrap().get(action).unwrap()
                );
            }
            println!();
        }
    }

    // fn save_result(&self) {
    //     let serialized = serde_json::to_string(&self.q).unwrap();

    //     let mut file = match File::create("model.json") {
    //         Ok(it) => it,
    //         Err(err) => return,
    //     };
    //     file.write_all(serialized.as_bytes()).unwrap();
    // }
}

impl Default for QLearner {
    fn default() -> Self {
        let game = Game::new();
        Self::new(game)
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "has_highest: {:?}", self.has_highest)?;
        writeln!(f, "can_follow: {}", self.can_follow)?;
        writeln!(f, "first: {}", self.first)?;
        writeln!(f, "first_suit: {}", self.first_suit)?;
        write!(f, "nb_trump: {}", self.nb_trump)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::PlayWorst => write!(f, "PlayWorst"),
            Action::RaiseLow => write!(f, "RaiseLow"),
            Action::RaiseHigh => write!(f, "RaiseHigh"),
            Action::TrumpHigh => write!(f, "TrumpHigh"),
            Action::TrumpLow => write!(f, "TrumpLow"),
            Action::Random => write!(f, "Random"),
            Action::PlayBest => write!(f, "PlayBest"),
            Action::ComeBest => write!(f, "ComeBest"),
        }
    }
}