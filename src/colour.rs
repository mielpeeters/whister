/*!
 * The four colours of cards: Spades, Clubs, Diamonds and Hearts.
 */

use std::fmt;
use std::slice::Iter;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Colour {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clr = match self {
            Colour::Spades => "\x1b[30;47m♠\x1b[0m",
            Colour::Clubs => "\x1b[30;47m♣\x1b[0m",
            Colour::Diamonds => "\x1b[91;47m♦\x1b[0m",
            Colour::Hearts => "\x1b[91;47m♥\x1b[0m",
        };

        write!(f, "{}", clr)
    }
}

impl Colour {
    pub fn iterator() -> Iter<'static, Colour> {
        static COLOURS: [Colour; 4] = [Colour::Spades, Colour::Clubs, Colour::Diamonds, Colour::Hearts];
        COLOURS.iter()
    }
}
