use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;
use std::cmp::Ordering;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Colour {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Eq, Clone)]
pub struct Card {
    pub colour: Colour,
    pub number: u8,
}

pub struct Deck {
    cards: Vec<Card>,
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

        Deck { cards }
    }

    pub fn new_empty() -> Self {
        let cards: Vec<Card> = Vec::new();
        Deck {
            cards
        }
    }

    fn get_amounts(cards: &Vec<Card>) -> Vec<usize> {
        let mut amounts: Vec<usize> = vec![0,0,0,0];
        
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
        Deck { cards }
    }

    /// Pull an amount of cards from the deck, in current deck order.
    pub fn pull_cards(&mut self, amount: usize) -> Deck {
        let pulled = self.cards.drain(..amount).collect();
        Deck { cards: pulled }
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
        let amounts = Self::get_amounts(&self.cards);

        let start = match colour {
            Colour::Spades => 0,
            Colour::Clubs => amounts[0],
            Colour::Diamonds => amounts[0] + amounts[1],
            Colour::Hearts => self.size() - amounts[3],
        };

        let cards: Vec<Card> = match colour {
            Colour::Spades => self.cards[start..start+amounts[0]].to_vec(),
            Colour::Clubs => self.cards[start..start+amounts[1]].to_vec(),
            Colour::Diamonds => self.cards[start..start+amounts[2]].to_vec(),
            Colour::Hearts => self.cards[start..start+amounts[3]].to_vec(),
        };

        Self::new_from(cards)
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    pub fn index_of(&self, card: &Card) -> Result<usize, usize> {
        self.cards.binary_search(card)
    }

    /// Remove the card at given index from this deck, and return ownership to caller.
    /// 
    /// This is useful when playing a game, and a player puts a card from its deck to the table's deck.
    pub fn remove(&mut self, index: usize) -> Card {
        self.cards.remove(index)
    }

    /// Add the given card to the deck. 
    /// 
    /// Useful for adding a card to the table's deck for example.
    /// 
    /// *Note: consumes the card!*
    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn show_sort(&mut self) {
        self.sort();

        println!("{}\n", self);
    }

    pub fn show(&self) {
        println!("{}\n", self);
    }

    /// look at one random card inside this deck
    pub fn peek(&self) -> Card {
        let random_index = rand::thread_rng().gen_range(0..self.size());
        self.cards[random_index].clone()
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
            nb = String::from("⒑");
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
        } else if self.number == 1 {
            Ordering::Greater
        } else if other.number == 1 {
            Ordering::Less
        } else {
            self.number.cmp(&other.number)
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
        
        // do not show an empty deck!
        if self.size() == 0 {
            return write!(f, "  Empty")
        }

        write!(f, "╭╴")?;
        
        let mut current_colour = self.cards[0].colour;

        for card in &self.cards {
            if card.colour != current_colour {
                write!(f, "\n│ ")?;
                current_colour = card.colour;
            }
            write!(f, "{}, ", card)?;
        }

        write!(f, "\n╰╴count: {}", self.size())
    }
}