/*!
 * This module implements some functions for showing the game state to the player.
 */

use std::{thread, time::Duration};

use crate::deck::Deck;

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

pub fn wait() {
    thread::sleep(Duration::from_millis(500));
}

pub fn show_last_non_empty(deck: &Vec<Deck>) {
    if !deck.is_empty() {
        println!("Played trick:\n{}\n",deck[deck.len()-1]);
    }
}

pub fn winner(player_id: usize) {
    println!("Winner this round: {}", match player_id {
        0 => "\x1b[1;92mYou!".to_string(),
        _ => format!("\x1b[1;91mPlayer {}", player_id),
    });
    clear_fmt();

    thread::sleep(Duration::from_millis(1500));
}