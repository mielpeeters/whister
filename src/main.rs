#![allow(dead_code)]
use rand::seq::SliceRandom;
use std::fmt;
use std::cmp::Ordering;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Colour {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Eq, Copy, Clone)]
struct Card {
    colour: Colour,
    number: u8,
}

struct Deck {
    cards: Vec<Card>,
    /// keeps the amount of cards of each colour type
    amounts: Vec<usize>,
}

struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    tricks: Vec<Deck>,
}

impl Deck {
    pub fn new_full() -> Self {
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

        let amounts: Vec<usize> = vec![13, 13, 13, 13];

        Deck { cards, amounts }
    }

    fn get_amounts(cards: &Vec<Card>) -> Vec<usize> {
        let mut amounts: Vec<usize> = Vec::new();
        amounts.push(0);
        amounts.push(0);
        amounts.push(0);
        amounts.push(0);
        
        // count the amounts
        for card in cards {
            match card.colour {
                Colour::Spades => amounts[0] += 1,
                Colour::Clubs => amounts[1] += 1,
                Colour::Diamonds => amounts[2] += 1,
                Colour::Hearts => amounts[3] += 1,
            }
        }

        amounts
    }

    pub fn new_from(cards: Vec<Card>) -> Deck {
        let amounts = Self::get_amounts(&cards);
        Deck { cards, amounts}
    }

    pub fn pull_cards(&mut self, amount: usize) -> Deck {
        let pulled = self.cards.drain(..amount).collect();
        let amounts = Self::get_amounts(&pulled);
        Deck { cards: pulled , amounts}
    }

    pub fn size(&self) -> usize {
        self.cards.len()
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::thread_rng());
    }

    /// sort the cards by colour first, then by ascending number
    pub fn sort(&mut self) {
        self.cards.sort();
    }

    pub fn get_deck_of_colour(&self, colour: Colour) -> Deck {
        let start = match colour {
            Colour::Spades => 0,
            Colour::Clubs => self.amounts[0],
            Colour::Diamonds => self.amounts[0] + self.amounts[1],
            Colour::Hearts => self.size() - self.amounts[3],
        };

        let cards: Vec<Card> = match colour {
            Colour::Spades => self.cards[start..start+self.amounts[0]].to_vec(),
            Colour::Clubs => self.cards[start..start+self.amounts[1]].to_vec(),
            Colour::Diamonds => self.cards[start..start+self.amounts[2]].to_vec(),
            Colour::Hearts => self.cards[start..start+self.amounts[3]].to_vec(),
        };

        Self::new_from(cards)
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
        // Ten
        else if nb == "10" {
            nb = String::from("T");
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

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.colour != other.colour {
            self.colour.cmp(&other.colour)
        } else {
            if self.number == 1 {
                Ordering::Greater
            } else if other.number == 1 {
                Ordering::Less
            } else {
                self.number.cmp(&other.number)
            }
        }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.colour == other.colour && self.number == other.number
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
