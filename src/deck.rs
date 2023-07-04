/*!
 * Pack of cards, not necessarily a full deck. Lots of functionality.
 */

use crate::{card::Card, suit::Suit, player::Player};
use rand::seq::SliceRandom;
use rand::Rng;
use std::{cmp::min, fmt};

pub type CardID = usize;

#[derive(Hash, Eq, PartialEq)]
pub struct Deck {
    pub cards: Vec<Card>,
    selected: CardID,
    suit_amounts: [usize; 4],
    curr: usize,
}

impl Deck {
    pub fn new_full() -> Self {
        let mut cards: Vec<Card> = Vec::new();

        let suits = [Suit::Hearts, Suit::Clubs, Suit::Diamonds, Suit::Spades];

        for clr in suits {
            for nmb in 1..14 {
                cards.push(Card {
                    suit: clr,
                    number: nmb,
                });
            }
        }

        Deck {
            cards,
            selected: usize::MAX,
            suit_amounts: [13, 13, 13, 13],
            curr: 0,
        }
    }

    pub fn new_empty() -> Self {
        let cards: Vec<Card> = Vec::new();
        let mut deck = Deck {
            cards,
            ..Default::default()
        };
        deck.set_suit_amounts();

        deck
    }

    pub fn card(&self, index: CardID) -> &Card {
        &self.cards[index % self.cards.len()]
    }

    pub fn has_suit(&self, suit: &Suit) -> bool {
        self.suit_amounts[*suit as usize] > 0
    }

    pub fn suit_of(&self, index: CardID) -> Suit {
        self.cards[index].suit
    }

    fn get_amounts(cards: &Vec<Card>) -> Vec<usize> {
        let mut amounts: Vec<usize> = vec![0, 0, 0, 0];

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
        let mut deck = Deck {
            cards,
            ..Default::default()
        };
        deck.set_suit_amounts();

        deck
    }

    /// Pull an amount of cards from the deck, in current deck order.
    pub fn pull_cards(&mut self, amount: usize) -> Deck {
        let pulled = self.cards.drain(..amount).collect();
        let mut deck = Deck {
            cards: pulled,
            ..Default::default()
        };

        // update both decks' suit amounts
        self.set_suit_amounts();
        deck.set_suit_amounts();

        // return the pulled cards
        deck
    }

    pub fn size(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::thread_rng());
    }

    /// sort the cards by Suits first, then by ascending number
    pub fn sort(&mut self) {
        self.cards.sort();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Card> {
        self.cards.iter()
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
            Suit::Spades => self.cards[start..start + amounts[0]].to_vec(),
            Suit::Clubs => self.cards[start..start + amounts[1]].to_vec(),
            Suit::Diamonds => self.cards[start..start + amounts[2]].to_vec(),
            Suit::Hearts => self.cards[start..start + amounts[3]].to_vec(),
        };

        Self::new_from(cards)
    }

    fn suit_score(&self, suit: &Suit) -> u32 {
        self.get_deck_of_suit(suit)
            .cards
            .iter()
            .map(Card::score)
            .sum()
    }

    pub fn best_suit_score(&self) -> (Suit, u32) {
        let best_suit = Suit::iterator()
            .enumerate()
            .max_by(|(_, suit_1), (_, suit_2)| {
                self.suit_score(suit_1).cmp(&self.suit_score(suit_2))
            })
            .map(|(_, suit)| suit)
            .unwrap();

        (*best_suit, self.suit_score(best_suit))
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    pub fn id_of(&self, card: &Card) -> Option<CardID> {
        self.cards.iter().position(|c| c == card)
    }

    /// Remove the card at given index from this deck, and return ownership to caller.
    ///
    /// This is useful when playing a game, and a player puts a card from its deck to the table's deck.
    pub fn remove(&mut self, index: CardID) -> Card {
        self.suit_amounts[self.cards[index].suit as usize] -= 1;
        if index == self.selected && index != 0 {
            self.selected -= 1;
        }
        self.cards.remove(index)
    }

    /// Add the given card to the deck.
    ///
    /// Useful for adding a card to the table's deck for example.
    ///
    /// *Note: consumes the card!*
    pub fn add(&mut self, card: Card) {
        self.suit_amounts[card.suit as usize] += 1;
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

    pub fn lowest(&self, available: &[CardID], trump: &Suit) -> Option<CardID> {
        available
            .iter()
            .enumerate()
            .min_by(|(_, me), (_, other)| self.card(**me).higher(self.card(**other), trump))
            .map(|(_, card_id)| card_id).copied()
    }

    pub fn highest(&self, available: &[CardID], trump: &Suit) -> Option<CardID> {
        available
            .iter()
            .enumerate()
            .max_by(|(_, one), (_, two)| self.card(**one).higher(self.card(**two), trump))
            .map(|(_, card_id)| card_id).copied()
    }

    pub fn winning(&self, available: &[CardID], trump: &Suit) -> Option<CardID> {
        available
            .iter()
            .enumerate()
            .max_by(|(_, one), (_, two)| self.card(**one).winning(self.card(**two), trump))
            .map(|(_, card_id)| card_id).copied()
    }

    fn set_suit_amounts(&mut self) {
        let mut amounts: [usize; 4] = [0; 4];

        self.cards
            .iter()
            .for_each(|c| amounts[c.suit as usize] += 1);

        self.suit_amounts = amounts;
    }

    pub fn get_suit_amount(&self, suit: &Suit) -> usize {
        self.suit_amounts[*suit as usize]
    }

    pub fn selected(&self) -> &Card {
        &self.cards[self.selected]
    }

    pub fn selected_id(&self) -> CardID {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected;
    }

    /// returns (y, x) where y is the suit index and x is the index within that suit
    fn selected_to_coordinate(&self) -> (usize, usize) {
        let suit = self.card(self.selected).suit;

        let y = suit as usize;

        let mut start: usize = 0;
        for s in 0..y {
            start += self.suit_amounts[s];
        }

        let x = self.selected - start;

        (y, x)
    }

    fn coordinate_to_selected(&mut self, coord: (usize, usize)) {
        let mut selected: usize = 0;

        for s in 0..coord.0 {
            selected += self.suit_amounts[s];
        }

        self.selected = selected + coord.1;
    }
}

impl Player for Deck {
    /// Create a new player, pulling `amount` cards from `deck`
    fn new_take_cards(deck: &mut Deck, amount: usize) -> Deck {
        let mut pulled = deck.pull_cards(amount);
        pulled.sort();
        Deck::new_from(pulled.cards)
    }
    
    /// show the cards this player is holding, by first sorting them!
    fn show_cards(&mut self) {
        self.show_sort();
    }
    
    /// look at a random card of this player's deck
    fn random_card(&self) -> &Card {
        self.peek()
    }
    
    /// Does this player have any cards left?
    fn has_cards(&self) -> bool {
        !self.is_empty()
    }
    
    /// Does this player have any cards of this suit?
    fn can_follow(&self, suit: Suit) -> bool {
        self.has_suit(&suit)
    }
    
    fn card(&self, card: CardID) -> &Card {
        self.card(card)
    }
    
    fn selected_card(&self) -> &Card {
        self.selected()
    }
    
    fn selected_id(&self) -> CardID {
        self.selected_id()
    }

    fn select_right(&mut self) {
        let coor = self.selected_to_coordinate();

        let mut new_x = coor.1 + 1;

        new_x %= self.suit_amounts[coor.0];

        self.coordinate_to_selected((coor.0, new_x));
    }

    fn select_left(&mut self) {
        let coor = self.selected_to_coordinate();

        if coor.1 == 0 {
            self.coordinate_to_selected((coor.0, self.suit_amounts[coor.0] - 1));
            return;
        }

        let mut new_x = coor.1 - 1;

        new_x %= self.suit_amounts[coor.0];

        self.coordinate_to_selected((coor.0, new_x));
    }

    fn select_up(&mut self) {
        let coor = self.selected_to_coordinate();

        let mut new_y = coor.0;
        // loop until another suit is found that has at least one card
        loop {
            new_y += 3;
            new_y %= 4;

            if self.suit_amounts[new_y] > 0 {
                break;
            }
        }

        let new_x = min(self.suit_amounts[new_y] - 1, coor.1);

        self.coordinate_to_selected((new_y, new_x));
    }

    fn select_down(&mut self) {
        let coor = self.selected_to_coordinate();

        let mut new_y = coor.0;
        // loop until another suit is found that has at least one card
        loop {
            new_y += 1;
            new_y %= 4;

            if self.suit_amounts[new_y] > 0 {
                break;
            }
        }

        let new_x = min(self.suit_amounts[new_y] - 1, coor.1);

        self.coordinate_to_selected((new_y, new_x));
    }
}

impl Default for Deck {
    fn default() -> Self {
        Deck::new_full()
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // do not show an empty deck!
        if self.is_empty() {
            return write!(f, "  Empty");
        }

        write!(f, "\x1b[2m╭╴\x1b[0m")?;

        let mut current_suit = self.cards[0].suit;

        for card in &self.cards {
            if card.suit != current_suit {
                write!(f, "\n\x1b[2m│\x1b[0m ")?;
                current_suit = card.suit;
            }

            // show card, selected card shown differently
            if self.id_of(card).unwrap_or(0) == self.selected {
                write!(f, "\x1b[7m{}, \x1b[0m", card)?;
            } else {
                write!(f, "{}, ", card)?;
            }
        }

        write!(f, "\n\x1b[2m╰╴count: {}\x1b[0m", self.size())
    }
}

impl Iterator for Deck {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.cards.len() {
            return None;
        }
        let current = &self.cards[self.curr];
        self.curr += 1;

        Some(current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_deck() -> Deck {
        let mut deck = Deck::new_full();
        deck.sort();
        deck
    }

    #[test]
    fn highest_one_suit() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        assert_eq!(deck.highest(available, &Suit::Hearts).unwrap(), 12);
    }

    #[test]
    fn highest_mult_suits() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 24];

        assert_eq!(deck.highest(available, &Suit::Hearts).unwrap(), 12);
    }

    #[test]
    fn highest_trump() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 12, 37];

        assert_eq!(deck.highest(available, &Suit::Hearts).unwrap(), 12);
    }

    #[test]
    fn lowest_one_suit() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        assert_eq!(deck.lowest(available, &Suit::Hearts).unwrap(), 0);
    }

    #[test]
    fn lowest_mult_suits() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 24];

        assert_eq!(deck.lowest(available, &Suit::Hearts).unwrap(), 0);
    }

    #[test]
    fn lowest_trump() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 12, 37];

        assert_eq!(deck.lowest(available, &Suit::Hearts).unwrap(), 0);
    }

    #[test]
    fn winning_one_suit() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        assert_eq!(deck.winning(available, &Suit::Hearts).unwrap(), 12);
    }

    #[test]
    fn winning_mult_suits() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 24];

        assert_eq!(deck.winning(available, &Suit::Hearts).unwrap(), 12);
    }

    #[test]
    fn winning_trump() {
        let deck = init_deck();

        let available: &[CardID] = &[0, 1, 2, 3, 12, 39];

        assert_eq!(deck.winning(available, &Suit::Hearts).unwrap(), 39);
    }

    #[test]
    fn suit_amounts_init() {
        let deck = init_deck();

        assert_eq!(deck.get_suit_amount(&Suit::Diamonds), 13);
    }

    #[test]
    fn suit_amounts_pull() {
        let mut deck = init_deck();
        let pulled = deck.pull_cards(12);

        assert_eq!(
            deck.get_suit_amount(&Suit::Diamonds),
            13 - pulled.get_suit_amount(&Suit::Diamonds)
        );
        assert_eq!(
            deck.get_suit_amount(&Suit::Hearts),
            13 - pulled.get_suit_amount(&Suit::Hearts)
        );
        assert_eq!(
            deck.get_suit_amount(&Suit::Spades),
            13 - pulled.get_suit_amount(&Suit::Spades)
        );
        assert_eq!(
            deck.get_suit_amount(&Suit::Clubs),
            13 - pulled.get_suit_amount(&Suit::Clubs)
        );
    }

    #[test]
    fn can_follow() {

    }
}
