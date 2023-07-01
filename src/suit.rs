/*!
 * The four suits of cards: Spades, Clubs, Diamonds and Hearts.
 */

use std::fmt;
use std::slice::Iter;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Suit {
    Spades = 0,
    Clubs = 1,
    Diamonds = 2,
    Hearts = 3,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clr = match self {
            Suit::Spades => "\x1b[30;47m♠\x1b[0m",
            Suit::Clubs => "\x1b[30;47m♣\x1b[0m",
            Suit::Diamonds => "\x1b[91;47m♦\x1b[0m",
            Suit::Hearts => "\x1b[91;47m♥\x1b[0m",
        };

        write!(f, "{}", clr)
    }
}

impl Suit {
    pub fn iterator() -> Iter<'static, Suit> {
        static SUITS: [Suit; 4] = [Suit::Spades, Suit::Clubs, Suit::Diamonds, Suit::Hearts];
        SUITS.iter()
    }
}
