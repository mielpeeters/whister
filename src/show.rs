/*!
 * This module implements some functions for showing the game state to the player.
 */

use std::{thread, time::Duration, io::{stdin, stdout, Write, self}};

use termion::{raw::IntoRawMode, input::TermRead, event::Key};

use crate::deck::Deck;

macro_rules! print_player {
    ($x:expr) => {   
        print!("{}", match $x {
            0 => "\x1b[1;92mYou!".to_string(),
            _ => format!("\x1b[1;91mPlayer {}", $x),
        });
        clear_fmt();
    };
}

pub fn clear() {
    println!("\x1b[1J\x1b[H");
}

fn clear_fmt() {
    print!("\x1b[0m");
}

pub fn show_table_wait(table: &Deck) {
    clear();
    println!("Current table: \n{}\n", table);
    thread::sleep(Duration::from_millis(500));
}

pub fn show_table(table: &Deck) {
    clear();
    println!("Current table: \n{}\n", table);
}

pub fn dealer(dealer: usize) {
    clear();
    print!("The current dealer is Player ");
    print_player!(dealer);
    println!("\n");
}

pub fn wait() {
    thread::sleep(Duration::from_millis(500));
}

pub fn show_last_non_empty(deck: &Vec<Deck>) {
    if !deck.is_empty() {
        println!("Played trick:\n{}\n",deck[deck.len()-1]);
    }
}

pub fn winner(player_id: usize) {
    print!("Winner this round: ");
    print_player!(player_id);
    println!("\n");
}

pub fn wait_q() {
    println!("\x1b[3mPress [Q] to continue.\x1b[0m");

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    stdout.flush().unwrap();

    for c in stdin.keys() {
        {
            if let Key::Char('q') = c.unwrap() {
                break;
            }
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

pub fn wait_any() {
    println!("\x1b[3mPress [any key] to continue.\x1b[0m");

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    stdout.flush().unwrap();

    // just wait for one key to be pressed
    if stdin.keys().next().is_some() {}

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

pub fn wait_enter() {
    println!("\x1b[3mPress [enter key] to continue.\x1b[0m");

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    stdout.flush().unwrap();

    for c in stdin.keys() {
        if let Key::Char('\n') = c.unwrap() {
            break;
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

/// returns `true` if user entered y, otherwise false
/// 
/// `default` is returned when neither y or n is entered
pub fn yes_or_no(default: bool) -> bool {
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            if n > 0 {
                if default {
                    !input.contains('n')
                } else {
                    input.contains('y')
                }
            } else {
                default
            }
        }
        Err(_) => default,
    }
}

fn without_return(input: &str) -> String {
    let split: Vec<&str> = input.split('\n').collect();
    String::from(split[0])
}

/// returns `true` if user entered y, otherwise false
/// 
/// `default` is returned when neither y or n is entered
pub fn get_answer() -> Option<String> {
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            if n > 0 {
                Some(without_return(input.as_str()))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
