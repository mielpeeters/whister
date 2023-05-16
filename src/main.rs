/*!
This crate implements an AI which plays Colour Whist (nl: Kleurenwiezen).
*/

use whister::playingcards::Deck;

fn main() {
    let mut deck = Deck::new_full();

    deck.shuffle();

    let mut player_one = deck.pull_cards(13);
    player_one.sort();
    println!("Player one's deck: {}\n", player_one);

    let mut player_two = deck.pull_cards(13);
    player_two.sort();
    println!("Player two's deck: {}\n", player_two);

    let mut player_three = deck.pull_cards(13);
    player_three.sort();
    println!("Player three's deck: {}\n", player_three);

    let mut player_four = deck.pull_cards(13);
    player_four.sort();
    println!("Player four's deck: {}\n", player_four);
}
