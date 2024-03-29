/*!
This module implements approximate Q-learning for the whister card game.

Usage:
```no_run
use whister::{
    fortify::{self, QLearner},
    game::Game,
    gamestate::GameState
    };

let mut learner: QLearner<GameState>;

// set the amount of iterations to a million
learner = QLearner::new_with_iter(1000000);

// you need to explicitely enable self-play
learner.enable_self_play();

learner.train(&mut Game::new());

let q = learner.get_q();

println!("resulting q values: {q:?}");
```
*/

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

pub mod data;

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
    + Send
    + Sync
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
        + for<'a> Deserialize<'a>
        + Send
        + Sync;
}

pub trait GameSpace<S: State>: Send + Sync {
    /// returns a new gamespace to learn in
    fn new_space(&self) -> Box<dyn GameSpace<S>>;

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
    fn take_action(&mut self, action: &S::A, q: &Option<&Q<S>>);
}

pub type Q<S> = HashMap<S, HashMap<<S as State>::A, f64>>;

/// determine the best action in current state, based on the q function
pub fn best_action_score<S: State>(q: &Q<S>, state: &S) -> Result<(S::A, f64), String> {
    let entry = q.get(state);

    if let Some(entry) = entry {
        if let Some((max_key, max_value)) = entry
            .iter()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(Ordering::Equal))
        {
            return Ok((*max_key, *max_value));
        }
    }

    Err("There was no entry for this state.".to_string())
}

pub struct QLearner<S>
where
    S: State,
{
    q: Q<S>,
    pub rate: f64,
    pub discount: f64,
    pub initial_value: f64,
    iterations: u64,
    current_iter: u64,
    self_play: bool,
    pub queue_size: usize,
    verbose: bool,
}

impl<S> QLearner<S>
where
    S: State + 'static,
{
    pub fn new() -> Self {
        let q = HashMap::new();

        QLearner {
            q,
            rate: 0.05,
            discount: 0.2,
            initial_value: 0.8,
            iterations: 100000,
            current_iter: 0,
            self_play: false,
            queue_size: 500,
            verbose: true,
        }
    }

    pub fn enable_self_play(&mut self) {
        self.self_play = true;
    }

    pub fn disable_verbose(&mut self) {
        self.verbose = false;
    }

    pub fn new_with_iter(iter: u64) -> Self {
        let mut learner = Self::new();

        if iter > learner.queue_size as u64 {
            learner.iterations = iter;
        } else {
            learner.iterations = learner.queue_size as u64;
        }

        learner
    }

    /// Train an agent in the given gamespace.
    /// The gamespace (`game` parameter) is only an example, other spaces will be instantiated in
    /// the multithreaded training process
    pub fn train(&mut self, game: &mut impl GameSpace<S>) {
        let pb = ProgressBar::new(self.iterations);
        pb.set_style(
            ProgressStyle::with_template("  {bar:40.green/black}  {pos} / {len}  eta: {eta}")
                .unwrap()
                .progress_chars("━━─"),
        );

        // create the channel
        let (tx, rx) = mpsc::channel();

        // keep track of the different handles
        let mut handles = Vec::new();

        // create a shared ownership Q
        let q = Arc::new(RwLock::new(self.q.clone()));

        let num_cpu = num_cpus::get();

        let producers = {
            if num_cpu < 2 {
                1
            } else {
                // use all but one of the available cpus
                // the last one should be kept free for the consumer
                num_cpu - 1
            }
        };

        let local_init = self.initial_value;
        let local_self = self.self_play;
        let local_disc = self.discount;

        for _ in 0..producers {
            // clone the tranceiver
            let local_tx = tx.clone();
            // create a new space to learn in
            let mut local_game = game.new_space();

            let q = Arc::clone(&q);

            let handle = thread::spawn(move || {
                loop {
                    let current_state = local_game.state();

                    // determine a new action to take, from current state
                    let action = local_game.random_action();

                    if local_self {
                        local_game.take_action(&action, &Some(&q.read().unwrap()));
                    } else {
                        local_game.take_action(&action, &None)
                    }

                    // reward is the reward that's coupled with this action
                    let reward = local_game.reward();
                    let best_future = {
                        let best = best_action_score(&q.read().unwrap(), &local_game.state());
                        if let Ok(best) = best {
                            best.1
                        } else {
                            local_init
                        }
                    };

                    // send the values to the consumer
                    let Ok(_) =
                        local_tx.send((current_state, action, reward + local_disc * best_future))
                    else {
                        break;
                    };
                }
            });
            handles.push(handle);
        }

        let mut rcv_queue = Vec::new();

        // consumer loop
        'outer: for rcv in rx {
            rcv_queue.push(rcv);

            if rcv_queue.len() == producers * self.queue_size {
                let mut my_q = q.write().unwrap();

                // consume the entire queue
                while let Some(rcv) = rcv_queue.pop() {
                    let (current_state, action, new) = rcv;

                    // new value to assign to Q(s,a)
                    let v: f64 = {
                        // get the old value of Q(s,a) if it is available
                        let old_value = my_q
                            .get(&current_state)
                            .and_then(|m| m.get(&action))
                            .unwrap_or(&self.initial_value);

                        *old_value + self.rate * (new - *old_value)
                    };

                    my_q.entry(current_state)
                        .or_insert_with(HashMap::new)
                        .insert(action, v);

                    self.current_iter += 1;

                    if self.verbose {
                        pb.inc(1);
                    }

                    if self.current_iter >= self.iterations {
                        break 'outer;
                    }
                }
            }
        }

        for handle in handles {
            handle.join().unwrap();
        }

        self.q = q.read().unwrap().clone();

        if self.verbose {
            pb.finish();
            println!(
                "\x1b[3m{} states\x1b[0m have been discovered",
                self.q.keys().len()
            );
            println!();
        }
    }

    pub fn get_q(&self) -> Q<S> {
        self.q.clone()
    }

    pub fn set_q(&mut self, q: Q<S>) {
        self.q = q;
    }
}

impl<S> Default for QLearner<S>
where
    S: State + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
