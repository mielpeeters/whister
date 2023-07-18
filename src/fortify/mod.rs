/*!
 * This module implements approximate Q-learning for the whister card game.
 *
 * Multiple difficulty levels are defined, by giving the agent increasing amounts of data to
 * train on. Trained models will be supplied when they are finished (basically a serialized Q hash map).
 */
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{mpsc, Arc, Mutex, RwLock};
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
        let result = entry
            .iter()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(Ordering::Equal));
        let result = result.unwrap();

        return Ok((*result.0, *result.1));
    }

    Err("There was no entry for this state.".to_string())
}

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
    self_play: bool,
    queue_size: usize,
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
            discount: 0.8,
            initial_value: 0.5,
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

    fn populate_mutex(&self, mutex: &mut HashMap<S, Mutex<()>>) {
        self.q.iter().for_each(|(state, _)| {
            mutex.insert(*state, Mutex::new(()));
        });
    }

    pub fn train(&mut self, game: &mut impl GameSpace<S>) {
        let pb = ProgressBar::new(self.iterations);
        pb.set_style(
            ProgressStyle::with_template("  {bar:40.green/black}  {pos} / {len}  eta: {eta}")
                .unwrap()
                .progress_chars("━━─"),
        );

        // create a mutex hashmap that follows the Q structure
        let mut mutex: HashMap<S, Mutex<()>> = HashMap::new();
        self.populate_mutex(&mut mutex);

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
                num_cpu - 1
            }
        };

        for _ in 0..producers {
            // clone the tranceiver
            let local_tx = tx.clone();
            // create a new space to learn in
            let mut local_game = game.new_space();

            let local_init = self.initial_value;
            let local_self = self.self_play;

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
                    let Ok(_) = local_tx.send((current_state, action, reward, best_future)) else {
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

                while let Some(rcv) = rcv_queue.pop() {
                    let (current_state, action, reward, best_future) = rcv;

                    // new value to assign to Q(s,a)
                    let v: f64 = {
                        // get the old value of Q(s,a) if it is available
                        let old_value = my_q
                            .get(&current_state)
                            .and_then(|m| m.get(&action))
                            .unwrap_or(&self.initial_value);

                        *old_value + self.rate * (reward + self.discount * best_future - *old_value)
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
        }
    }

    /// determine the action the agent takes while exploring the statespace
    // fn new_action(&mut self, game: &dyn GameSpace<S>) -> S::A {
    //     let alowed = game.actions();

    //     let exploit_factor = rand::thread_rng().gen_range(0..100);

    //     let explore_factor: f64 =
    //         100.0 * (self.iterations as f64 - self.current_iter as f64) / (self.iterations as f64);

    //     let best = best_action_score(&self.q, &game.state());

    //     if let Ok(best) = best {
    //         if alowed.contains(&best.0) && exploit_factor > max(explore_factor as u64, 50) {
    //             // EXPLOIT
    //             return best.0;
    //         }
    //     }

    //     // EXPLORE
    //     game.random_action()
    // }

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
