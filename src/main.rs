#![allow(dead_code)]
use rand::seq::SliceRandom;
use std::fmt;

#[derive(Clone, Copy)]
enum Colour {
    Hearts,
    Spades,
    Diamonds,
    Clubs,
}

struct Card {
    colour: Colour,
    number: u8,
}

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn pull_cards(&mut self, amount: usize) -> Deck {
        let pulled = self.cards.drain(..amount).collect();
        Deck { cards: pulled }
    }

    fn size(&self) -> usize {
        self.cards.len()
    }

    fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::thread_rng());
    }

    /// sort the cards by colour first, then by ascending number
    fn sort(&mut self) {
        // TODO
    }
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clr = match self {
            Colour::Hearts => "♥",
            Colour::Spades => "♠",
            Colour::Clubs => "♣",
            Colour::Diamonds => "♦",
        };
        write!(f, "{}", clr)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nb = self.number.to_string();

        // special numbers
        // Ace
        if nb == "1" {
            nb = String::from("A");
        }
        // Jack
        else if nb == "11" {
            nb = String::from("J");
        }
        // Queen
        else if nb == "12" {
            nb = String::from("Q");
        }
        // King
        else if nb == "13" {
            nb = String::from("K");
        }

        write!(f, "{}{}", nb, self.colour)
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;

        let mut row_count = 0;

        for card in &self.cards {
            if row_count % 4 == 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}, ", card)?;
            row_count += 1;
        }

        write!(f, "\n], length: {}", self.size())
    }
}

fn full_deck() -> Deck {
    let mut cards: Vec<Card> = Vec::new();

    let colours = [
        Colour::Hearts,
        Colour::Clubs,
        Colour::Diamonds,
        Colour::Spades,
    ];

    for clr in colours {
        for nmb in 1..14 {
            cards.push(Card {
                colour: clr,
                number: nmb,
            });
        }
    }

    Deck { cards }
}

fn main() {
    let mut deck = full_deck();

    deck.shuffle();

    let player_one = deck.pull_cards(13);
    println!("Player one's deck: {}\n", player_one);

    let player_two = deck.pull_cards(13);
    println!("Player two's deck: {}\n", player_two);

    let player_three = deck.pull_cards(13);
    println!("Player three's deck: {}\n", player_three);

    let player_four = deck.pull_cards(13);
    println!("Player four's deck: {}\n", player_four);
}
