/*!
 * Pack of cards, not necessarily a full deck. Lots of functionality.
 */

use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt;
use crate::{
    card::Card,
    colour::Colour,
};

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
    
    pub fn card(&self, index: usize) -> &Card {
        &self.cards[index]
    }

    pub fn has_colour(&self, colour: &Colour) -> bool {
        self.get_deck_of_colour(colour).cards.len() > 0
    }

    pub fn colour_at(&self, index: usize) -> Colour {
        self.cards[index].colour
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

    pub fn get_deck_of_colour(&self, colour: &Colour) -> Deck {
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

    fn colour_score(&self, colour: &Colour) -> u32 {
        let colour_deck = self.get_deck_of_colour(colour);

        let mut score: u32 = 0;
        for card in colour_deck.cards {
            score += card.score();
        }

        score
    }

    pub fn best_colour_score(&self) -> (Colour, u32) {
        let mut best_colour: Colour = Colour::Hearts;
        let mut best_score: u32 = 0;
        let mut current_score: u32;
        
        for colour in Colour::iterator() {
            current_score = self.colour_score(colour);
            if current_score > best_score {
                best_score = current_score; // copy
                best_colour = *colour;
            }
        }

        (best_colour, best_score)
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    pub fn index_of(&self, card: &Card) -> Option<usize> {
        self.cards.iter().position(|c| c == card)
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

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        // do not show an empty deck!
        if self.size() == 0 {
            return write!(f, "  Empty")
        }

        write!(f, "\x1b[2m╭╴\x1b[0m")?;
        
        let mut current_colour = self.cards[0].colour;

        for card in &self.cards {
            if card.colour != current_colour {
                write!(f, "\n\x1b[2m│\x1b[0m ")?;
                current_colour = card.colour;
            }
            write!(f, "{}, ", card)?;
        }

        write!(f, "\n\x1b[2m╰╴count: {}\x1b[0m", self.size())
    }
}