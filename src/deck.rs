/*!
 * Pack of cards, not necessarily a full deck. Lots of functionality.
 */

use rand::Rng;
use rand::seq::SliceRandom;
use std::{fmt, collections::HashMap, cmp::Ordering};
use crate::{
    card::Card,
    suit::Suit,
};

#[derive(Hash, Eq, PartialEq)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new_full() -> Self {
        let mut cards: Vec<Card> = Vec::new();

        let suits = [
            Suit::Hearts,
            Suit::Clubs,
            Suit::Diamonds,
            Suit::Spades,
        ];

        for clr in suits {
            for nmb in 1..14 {
                cards.push(Card {
                    suit: clr,
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

    pub fn has_suit(&self, suit: &Suit) -> bool {
        !self.get_deck_of_suit(suit).cards.is_empty()
    }

    pub fn suit_at(&self, index: usize) -> Suit {
        self.cards[index].suit
    }

    fn get_amounts(cards: &Vec<Card>) -> Vec<usize> {
        let mut amounts: Vec<usize> = vec![0,0,0,0];
        
        // count the amounts
        for card in cards {
            match card.suit {
                Suit::Spades => amounts[0] += 1,
                Suit::Clubs => amounts[1] += 1,
                Suit::Diamonds => amounts[2] += 1,
                Suit::Hearts => amounts[3] += 1,
            }
        }

        amounts
    }

    /// creates a new deck from these cards, consumes the cards.
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

    /// sort the cards by Suits first, then by ascending number
    pub fn sort(&mut self) {
        self.cards.sort();
    }

    pub fn get_deck_of_suit(&self, suit: &Suit) -> Deck {
        let amounts = Self::get_amounts(&self.cards);

        let start = match suit {
            Suit::Spades => 0,
            Suit::Clubs => amounts[0],
            Suit::Diamonds => amounts[0] + amounts[1],
            Suit::Hearts => self.size() - amounts[3],
        };

        let cards: Vec<Card> = match suit {
            Suit::Spades => self.cards[start..start+amounts[0]].to_vec(),
            Suit::Clubs => self.cards[start..start+amounts[1]].to_vec(),
            Suit::Diamonds => self.cards[start..start+amounts[2]].to_vec(),
            Suit::Hearts => self.cards[start..start+amounts[3]].to_vec(),
        };

        Self::new_from(cards)
    }

    fn suit_score(&self, suit: &Suit) -> u32 {
        self.get_deck_of_suit(suit).cards.iter()
                        .map(Card::score)
                        .sum()
    }

    pub fn best_suit_score(&self) -> (Suit, u32) {
        let best_suit = Suit::iterator()
            .enumerate()
            .max_by(|(_,suit_1),(_,suit_2)| {
                self.suit_score(suit_1).cmp(&self.suit_score(suit_2))
            })
            .map(|(_,suit)| suit)
            .unwrap();

        (*best_suit, self.suit_score(best_suit))
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

    /// Look at one random card inside this deck
    pub fn peek(&self) -> &Card {
        let random_index = rand::thread_rng().gen_range(0..self.size());
        &self.cards[random_index]
    }

    pub fn lowest(&self, available: &[usize], trump: &Suit) -> usize {
        *available
            .iter()
            .enumerate()
            .min_by(|(_, me),(_, other)| {
                if self.card(**me).better(self.card(**other), trump) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .map(|(_, index) | index)
            .unwrap()
    }

    pub fn highest(&self, available: &[usize], trump: &Suit) -> usize {
        *available
            .iter()
            .enumerate()
            .max_by(|(_, one), (_, two)| {
                if self.card(**one).better(self.card(**two), trump) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .map(|(_, i)| i)
            .unwrap()
    }

    pub fn suit_amounts(&self) -> HashMap<Suit, usize> {
        let mut map: HashMap<Suit, usize> = HashMap::new();

        for suit in Suit::iterator() {
            map.insert(*suit, 0);
        }

        let mut amnt: usize;
        for card in &self.cards {
            amnt = map.get(&card.suit).copied().unwrap_or(0);
            map.insert(card.suit, amnt+1);
        }

        map
    }

}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        // do not show an empty deck!
        if self.size() == 0 {
            return write!(f, "  Empty")
        }

        write!(f, "\x1b[2m╭╴\x1b[0m")?;
        
        let mut current_suit = self.cards[0].suit;

        for card in &self.cards {
            if card.suit != current_suit {
                write!(f, "\n\x1b[2m│\x1b[0m ")?;
                current_suit = card.suit;
            }
            write!(f, "{}, ", card)?;
        }

        write!(f, "\n\x1b[2m╰╴count: {}\x1b[0m", self.size())
    }
}