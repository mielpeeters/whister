/*!
 * This module implements approximate Q-learning for the whister card game.
 *
 * Multiple difficulty levels are defined, by giving the agent increasing amounts of data to
 * train on. Trained models will be supplied when they are finished (basically a serialized Q hash map).
 */
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read, Write};

pub trait State:
    PartialEq
    + Eq
    + Hash
    + Clone
    + Copy
    + Debug
    + PartialOrd
    + Ord
    + Serialize
    + for<'a> Deserialize<'a>
{
    /// the action type associated with this state
    type A: PartialEq
        + Eq
        + Hash
        + Clone
        + Copy
        + Debug
        + PartialOrd
        + Ord
        + Serialize
        + Default
        + for<'a> Deserialize<'a>;
}

pub trait GameSpace<S: State> {
    /// determine the reward coupled with getting to the current state
    fn reward(&self) -> f64;

    /// alowed actions from the current state
    fn actions(&self) -> Vec<S::A>;

    /// returns the current gamestate
    fn state(&self) -> S;

    /// gets a random alowed action (for exploration)
    fn random_action(&self) -> S::A {
        let actions = self.actions();
        let a_t = rand::random::<usize>() % actions.len();
        actions[a_t]
    }

    /// performs the action
    fn take_action(&mut self, action: &S::A);
}

pub type Q<S> = HashMap<S, HashMap<<S as State>::A, f64>>;

pub struct QLearner<S>
where
    S: State,
{
    q: Q<S>,
    rate: f64,
    discount: f64,
    initial_value: f64,
    iterations: u64,
    current_iter: u64,
}

impl<S> QLearner<S>
where
    S: State,
{
    pub fn new() -> Self {
        let q = HashMap::new();

        QLearner {
            q,
            rate: 0.05,
            discount: 0.8,
            initial_value: 0.5,
            iterations: 100000,
            current_iter: 0,
        }
    }

    pub fn new_with_iter(iter: u64) -> Self {
        let mut learner = Self::new();
        learner.iterations = iter;
        learner
    }

    pub fn train(&mut self, game: &mut dyn GameSpace<S>) {
        let pb = ProgressBar::new(self.iterations);
        pb.set_style(
            ProgressStyle::with_template("  {bar:40.green/black}   {pos} / {len}   eta: {eta}")
                .unwrap()
                .progress_chars("━━─"),
        );

        loop {
            let current_state = game.state();

            // determine a new action to take, from current state
            let action = self.new_action(game);

            game.take_action(&action);

            // reward is the reward that's coupled with this action
            let reward = game.reward();
            let best_future = *self.best_action_score(&game.state()).1;

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

            self.current_iter += 1;
            pb.inc(1);

            if self.current_iter >= self.iterations {
                break;
            }
        }

        pb.finish();
    }

    /// determine the best action in current state, based on the q function
    pub fn best_action_score(&mut self, state: &S) -> (&S::A, &f64) {
        let best = self
            .q
            .entry(*state)
            .or_insert_with(|| {
                let mut new = HashMap::new();
                new.insert(S::A::default(), self.initial_value);
                new
            })
            .iter()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(Ordering::Equal));

        best.unwrap()
    }

    /// determine the action the agent takes while exploring the statespace
    fn new_action(&mut self, game: &dyn GameSpace<S>) -> S::A {
        let alowed = game.actions();

        let exploit_factor = rand::thread_rng().gen_range(0..100);

        let explore_factor: f64 =
            100.0 * (self.iterations as f64 - self.current_iter as f64) / (self.iterations as f64);

        let best = self.best_action_score(&game.state());

        if alowed.contains(best.0) && exploit_factor > max(explore_factor as u64, 50) {
            // EXPLOIT
            return *best.0;
        }

        // EXPLORE
        game.random_action()
    }

    fn q_to_optimal(&self) -> HashMap<S, S::A> {
        let mut optimal_action = HashMap::new();
        self.q.iter().for_each(|test| {
            optimal_action.insert(
                *test.0,
                *self
                    .q
                    .get(test.0)
                    .unwrap()
                    .iter()
                    .max_by(|score1, score2| score1.1.partial_cmp(score2.1).unwrap())
                    .unwrap()
                    .0,
            );
        });

        optimal_action
    }

    fn optimal_to_q(&mut self, optimal: HashMap<S, S::A>) {
        self.q = HashMap::new();

        optimal.iter().for_each(|state_action| {
            let mut action_value = HashMap::new();
            action_value.insert(*state_action.1, 10.0);
            self.q.insert(*state_action.0, action_value);
        });
    }

    pub fn save_result(&self, path: String, reduced: bool) {
        let serialized = match reduced {
            true => {
                let optimal = self.q_to_optimal();
                serde_pickle::to_vec(&optimal, Default::default()).unwrap()
            }
            false => serde_pickle::to_vec(&self.q, Default::default()).unwrap(),
        };

        let mut file = match File::create(path) {
            Ok(it) => it,
            Err(_) => return,
        };

        file.write_all(serialized.as_slice()).unwrap();
    }

    pub fn import_from_model(&mut self, path: String, reduced: bool) {
        let file = match File::open(path) {
            Ok(it) => it,
            Err(_) => {
                println!("\x1b[91mCouldn't import that model...\x1b[0m");
                return;
            }
        };

        let mut reader = BufReader::new(file);
        let mut serialized = Vec::new();

        reader.read_to_end(&mut serialized).unwrap();

        if reduced {
            let deserialized: HashMap<S, S::A> =
                serde_pickle::from_slice(&serialized, Default::default()).unwrap();

            self.optimal_to_q(deserialized);
        } else {
            let deserialized: Q<S> =
                serde_pickle::from_slice(&serialized, Default::default()).unwrap();
            self.q = deserialized;
        }
    }
}

impl<S> Default for QLearner<S>
where
    S: State,
{
    fn default() -> Self {
        Self::new()
    }
}
