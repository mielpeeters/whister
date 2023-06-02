/*!
 * Playing card, meaning a suit and a number/score. 
 */

use crate::suit::Suit;
use std::cmp::Ordering;
use std::fmt;

#[derive(Eq, Clone)]
pub struct Card {
    pub suit: Suit,
    pub number: u8,
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.suit != other.suit {
            self.suit.cmp(&other.suit)
        } else if self.number == 1 {
            Ordering::Greater
        } else if other.number == 1 {
            Ordering::Less
        } else {
            self.number.cmp(&other.number)
        }
        
    }
}

impl Card {
    pub fn score(&self) -> u32 {
        if self.number == 1 {
            14
        } else {
            self.number.into()
        }
    }

    pub fn better(&self, other: &Card, trump: &Suit) -> bool {
        if self.suit == other.suit {
            return self > other;
        } else {
            if self.suit == *trump {
                return true;
            } else {
                return false;
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
        self.suit == other.suit && self.number == other.number
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


        write!(f, "\x1b[47;30m{}{}", nb, self.suit)
    }
}