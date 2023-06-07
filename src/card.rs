/*!
 * Playing card, meaning a suit and a number/score.
 */

use crate::suit::Suit;
use std::cmp::Ordering;
use std::fmt;

#[derive(Eq, Clone, PartialEq, Hash)]
pub struct Card {
    pub suit: Suit,
    pub number: u8,
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
            self > other
        } else {
            self.suit == *trump
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.suit != other.suit {
            self.suit.cmp(&other.suit)
        } else {
            self.score().cmp(&other.score())
        }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nb = self.number.to_string();

        // special numbers
        nb = match nb.as_str() {
            "1" => "A".to_string(),
            "10" => "T".to_string(),
            "11" => "J".to_string(),
            "12" => "D".to_string(),
            "13" => "K".to_string(),
            &_ => nb,
        };

        write!(f, "\x1b[47;30m{}{}", nb, self.suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn cmp_same_suit() {
        let ace_hearts = Card {
            suit: Suit::Hearts,
            number: 1,
        };

        let king_hearts = Card {
            suit: Suit::Hearts,
            number: 13,
        };

        assert!(ace_hearts > king_hearts);
    }

    #[test]
    fn cmp_different_suit() {
        let ace_clubs = Card {
            suit: Suit::Clubs,
            number: 1,
        };

        let king_hearts = Card {
            suit: Suit::Hearts,
            number: 13,
        };

        assert!(ace_clubs < king_hearts);
    }
}
