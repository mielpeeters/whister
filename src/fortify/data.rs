use dirs::data_dir;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{stdin, stdout, BufReader, Read, Write},
    process::exit,
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

use crate::show;

use super::State;
use super::Q;

pub fn q_to_optimal<S: State>(q: &Q<S>) -> HashMap<S, S::A> {
    let mut optimal_action = HashMap::new();
    q.iter().for_each(|test| {
        optimal_action.insert(
            *test.0,
            *q.get(test.0)
                .unwrap()
                .iter()
                .max_by(|score1, score2| score1.1.partial_cmp(score2.1).unwrap())
                .unwrap()
                .0,
        );
    });

    optimal_action
}

fn optimal_to_q<S: State>(optimal: HashMap<S, S::A>) -> Q<S> {
    let mut q = HashMap::new();

    optimal.iter().for_each(|state_action| {
        let mut action_value = HashMap::new();
        action_value.insert(*state_action.1, 10.0);
        q.insert(*state_action.0, action_value);
    });

    q
}

fn get_data_dir() -> Result<std::path::PathBuf, ()> {
    let mut data_dir = data_dir().expect("Could not find a data directory");
    data_dir.push("whister");

    if !data_dir.exists() {
        if fs::create_dir_all(&data_dir).is_ok() {
            Ok(data_dir)
        } else {
            Err(())
        }
    } else {
        Ok(data_dir)
    }
}

fn list_data() -> Vec<String> {
    let data_dir = get_data_dir().expect("Should get data directory");
    let paths = fs::read_dir(data_dir).unwrap();

    let mut models = Vec::new();
    for path in paths {
        let what = path.unwrap().file_name();
        let model_file_name = what.to_str().unwrap();
        let model_file_name_split: Vec<&str> = model_file_name.split('.').collect();
        models.push(String::from(model_file_name_split[0]));
    }

    models
}

fn get_save_file(file_name: &str) -> Result<std::fs::File, ()> {
    let mut data_dir = get_data_dir()?;

    // create the file path
    data_dir.push(format!("{}.bin", file_name));

    // create the file
    let file = File::create(data_dir).map_err(|_| ())?;

    // return file
    Ok(file)
}

fn get_data(file_name: &str) -> Option<Vec<u8>> {
    let mut data_dir = get_data_dir().expect("Should get data directory");

    // create the file path
    data_dir.push(format!("{}.bin", file_name));

    let file = match File::open(data_dir) {
        Ok(it) => it,
        Err(_) => return None,
    };

    let mut reader = BufReader::new(file);
    let mut serialized = Vec::new();

    reader.read_to_end(&mut serialized).unwrap();

    Some(serialized)
}

pub fn q_to_bin<S: State>(q: &Q<S>, name: String, reduced: bool) -> std::io::Result<()> {
    let serialized = match reduced {
        true => {
            let optimal = q_to_optimal(q);
            bincode::serialize(&optimal).expect("Should serialize reduced Q")
        }
        false => bincode::serialize(q).expect("Should serialize unreduced Q"),
    };

    let mut encoder = ZlibEncoder::new(get_save_file(name.as_str()).expect("Should get save file"), Compression::best());
    encoder.write_all(&serialized).unwrap();

    encoder.finish().unwrap();
    Ok(())
}

pub fn bin_to_q<S: State>(name: &str, reduced: bool) -> Option<Q<S>> {
    let Some(serialized) = get_data(name) else {
        return None
    };

    let mut decoder = ZlibDecoder::new(serialized.as_slice());
    let mut uncompressed: Vec<u8> = Vec::with_capacity(serialized.len());

    decoder.read_to_end(&mut uncompressed).unwrap();

    if reduced {
        let deserialized: HashMap<S, S::A> = bincode::deserialize(&uncompressed).expect("Should deserialize reduced Q");

        Some(optimal_to_q(deserialized))
    } else {
        let deserialized: Q<S> = bincode::deserialize(&uncompressed).expect("Should deserialize unreduced Q");
        Some(deserialized)
    }
}

fn show_selected_model(models: &Vec<String>, selected: usize, new: bool) {
    show::clear();
    println!("Select the saved AI model you want to use");

    for (current, model) in models.iter().enumerate() {
        if current == selected {
            println!("- \x1b[7m{}\x1b[0m", model);
        } else {
            println!("- {}", model);
        }
    }

    // also show the option to create a new model
    if new {
        if selected == models.len() {
            print!("\x1b[7m");
        }

        println!("\x1b[92;3mcreate new\x1b[0m");
    }

    println!();
}

fn ask_model(new: bool) -> Option<String> {
    let models = list_data();

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    stdout.flush().unwrap();

    let mut selected: usize = 0;

    stdout.suspend_raw_mode().unwrap();
    show_selected_model(&models, selected, new);
    stdout.activate_raw_mode().unwrap();

    for c in stdin.keys() {
        {
            match c.unwrap() {
                Key::Char('j') | Key::Down => {
                    selected += 1;
                    if new {
                        selected %= models.len() + 1;
                    } else {
                        selected %= models.len();
                    }
                }
                Key::Char('k') | Key::Up => {
                    if new {
                        selected += models.len();
                        selected %= models.len() + 1;
                    } else {
                        selected += models.len() - 1;
                        selected %= models.len();
                    }
                }
                Key::Char(' ') | Key::Char('\n') => break,
                Key::Char('q') => {
                    stdout.flush().unwrap();
                    write!(stdout, "{}", termion::cursor::Show).unwrap();
                    drop(stdout);
                    exit(0);
                }
                _ => {}
            }
        }
        stdout.flush().unwrap();
        stdout.suspend_raw_mode().unwrap();

        // show current selected
        show_selected_model(&models, selected, new);

        stdout.activate_raw_mode().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();

    if selected < models.len() {
        Some(models[selected].clone())
    } else {
        None
    }
}

/// Use terminal inputs to select one of the available models
///
/// - `new`: set to true if you want to allow the option of creating a new model!
pub fn select_model<S: State>(new: bool) -> Option<Q<S>> {
    let model = ask_model(new)?;

    bin_to_q(model.as_str(), false)
}
