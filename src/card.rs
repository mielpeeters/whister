/*!
 * Playing card, meaning a suit and a number/score.
 */

use crate::suit::Suit;
use std::cmp::Ordering;
use std::fmt;

/**
A playing card, meaning a suit and a number

*note: the numbering is 1..14 (14 not included), where 1 represents an Ace,
which is higher in value than any other number*

different types of ordering are provided, which can be used in
different situations:

```
# use whister::suit::Suit;
# use std::cmp::Ordering;
# use whister::card::Card;
let seven_of_spades = Card {suit: Suit::Spades, number: 7};
let king_of_clubs = Card {suit: Suit::Clubs, number: 1};

// using the `winning` comparison with hearts as trump,
// we see that the seven of spades wins.
// this resembles the situation where the seven is already on the table,
// and the king is placed on it,
// which means that player can't follow in spades.
let order = seven_of_spades.winning(&king_of_clubs, &Suit::Hearts);
assert!(order == Ordering::Greater);

// using the `higher` comparison with hearts as trump,
// we see that the seven of spades is lower than the king.
// this resembles the situation where player is comparing their own cards,
// to see which might be more valuable.
let order = seven_of_spades.higher(&king_of_clubs, &Suit::Hearts);
assert!(order == Ordering::Less);
```
*/
#[derive(Eq, Clone, PartialEq, Hash, Debug)]
pub struct Card {
    pub suit: Suit,
    pub number: u8,
}

impl Card {
    /// Returns the value of the number of this card (to that Ace > King > ...)
    pub fn score(&self) -> u32 {
        // elevate the score of an ace such that it is more powerful than a king
        if self.number == 1 {
            14
        } else {
            self.number.into()
        }
    }

    /// Determine which card wins when comparing them on a table
    pub fn winning(&self, other: &Card, trump: &Suit) -> Ordering {
        if self.suit == other.suit {
            // if we the cards have the same suit, we compare based on score
            self.score().cmp(&other.score())
        } else if self.suit == *trump {
            // if my suit is trump, and the other's isn't, I win
            Ordering::Greater
        } else if other.suit == *trump {
            // if my suit isn't trump, and the other's is, they win
            Ordering::Less
        } else {
            // if we have different non-trump suits, the first card dominates
            Ordering::Greater
        }
    }

    /// Determine which card has a higher "value" to the player
    pub fn higher(&self, other: &Card, trump: &Suit) -> Ordering {
        if self.suit != other.suit && self.suit != *trump && other.suit != *trump {
            // different suits, and none is trump
            self.score().cmp(&other.score())
        } else {
            self.winning(other, trump)
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
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
            "12" => "Q".to_string(),
            "13" => "K".to_string(),
            &_ => nb,
        };

        write!(f, "\x1b[47;30m{}{}", nb, self.suit)
    }
}

impl Default for Card {
    fn default() -> Self {
        Card {
            suit: Suit::Hearts,
            number: 1,
        }
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

    #[test]
    fn winning_same_suit() {
        let ace_hearts = Card {
            suit: Suit::Hearts,
            number: 1,
        };

        let king_hearts = Card {
            suit: Suit::Hearts,
            number: 13,
        };

        assert!(ace_hearts.winning(&king_hearts, &Suit::Hearts) == Ordering::Greater);
    }

    #[test]
    fn winning_different_suit_none_trump() {
        let ace_hearts = Card {
            suit: Suit::Hearts,
            number: 1,
        };

        let ace_clubs = Card {
            suit: Suit::Clubs,
            number: 13,
        };

        assert!(ace_hearts.winning(&ace_clubs, &Suit::Spades) == Ordering::Greater);
    }

    #[test]
    fn winning_different_suit_trump() {
        let ace_hearts = Card {
            suit: Suit::Hearts,
            number: 1,
        };

        let ace_clubs = Card {
            suit: Suit::Clubs,
            number: 13,
        };

        assert!(ace_hearts.winning(&ace_clubs, &Suit::Clubs) == Ordering::Less);
    }
}
