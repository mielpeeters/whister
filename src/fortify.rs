/*!
 * This module implements approximate Q-learning for the whister card game.
 *
 * Multiple difficulty levels are defined, by giving the agent increasing amounts of data to
 * train on. Trained models will be supplied when they are finished (basically a serialized Q hash map).
 */
#![allow(unused_variables)]

use crate::game::Game;
use std::{collections::HashMap, cmp::Ordering};


/// All possible actions that the agent can take.
/// Technically, the action could be described as just a Card, but
/// these are realistic moves for many scenarios, to reduce the state-action space.
#[derive(Hash, PartialEq, Eq)]
pub enum Action {
    /// play the lowest card you can
    PlayWorst,
    /// play a higher card, but the lowest you can
    RaiseLow,
    /// play a higher card, the highest you can
    RaiseHigh,
    /// play the highest trump card you can
    BuyHigh,
    /// play the lowest trump card that buys it
    BuyLow,
}

#[derive(PartialEq, Eq, Hash, Clone, Default, Copy)]
pub struct GameState {
    
}

impl GameState {
    pub fn new() -> GameState {
        GameState {}
    }
}

type Q = HashMap<GameState, HashMap<Action, f64>>;

pub struct QLearner {
    q: Q,
    rate: f64,
    discount: f64,
    initial_value: f64,
    iterations: u64,
}

impl QLearner {
    pub fn new() -> Self {
        let q = HashMap::new();    
        QLearner { q, rate: 0.2, discount: 0.05, initial_value: 0.0, iterations: 10000 }
    }
    pub fn train(&mut self, game: &mut Game) {
        let mut count = 0;
        loop {
            let current_state = game.state();
            let current_values = self.q.get(&current_state).unwrap_or(&HashMap::new());
            
            // determine a new action to take, from current state
            let action = self.new_action(&current_state);
            
            self.take_action(game, &action);
            
            // reward is the reward that's coupled with this action
            let reward = game.reward() as f64;
            let best_future = *self.best_action_score(&game.state()).1;
            
            // new value to assign to Q(s,a)
            let v: f64 = {
                // get the old value of Q(s,a) if it is available
                let old_value = self.q.get(&current_state).and_then(|m| m.get(&action)).unwrap_or(&self.initial_value);
                
                *old_value + self.rate * (reward + self.discount * best_future - *old_value)
            };
            
            self.q
            .entry(current_state)
            .or_insert_with(HashMap::new)
            .insert(action, v);
            
            count += 1;

            if count > self.iterations {
                break
            }
        }
    }

    /// determine the best action in current state, based on the q function
    pub fn best_action_score(&mut self, state: &GameState) -> (&Action, &f64) {
        let best = self.q
            .entry(*state)
            .or_insert_with(HashMap::new)
            .iter()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(Ordering::Equal));

        best.unwrap_or((&Action::PlayWorst, &0.0))
    }

    /// determine the action the agent takes while exploring the statespace
    fn new_action(&self, state: &GameState) -> Action {
        // TODO
        Action::PlayWorst
    }

    fn take_action(&self, game: &mut Game, action: &Action) {
        let playable = game.alowed_cards(0);

        match action {
            Action::PlayWorst => {
                let worst = game.lowest_card_of(0, &playable);
                game.agent_plays_round(worst);
            },
            Action::RaiseLow => todo!(),
            Action::RaiseHigh => todo!(),
            Action::BuyHigh => todo!(),
            Action::BuyLow => todo!(),
        }
    }
}

impl Default for QLearner {
    fn default() -> Self {
        Self::new()
    }
}
