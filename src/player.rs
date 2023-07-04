/*!
 * Player of the colour whist game.
 */

use crate::{
    card::Card,
    deck::{CardID, Deck},
    suit::Suit,
};

pub trait Player {
    /// Create a new player, pulling `amount` cards from `deck`
    fn new_take_cards(deck: &mut Deck, amount: usize) -> Self;

    /// show the cards this player is holding, by first sorting them!
    fn show_cards(&mut self);

    /// look at a random card of this player's deck
    fn random_card(&self) -> &Card ;

    /// Does this player have any cards left?
    fn has_cards(&self) -> bool;

    /// Does this player have any cards of this suit?
    fn can_follow(&self, suit: Suit) -> bool ;

    fn card(&self, card: CardID) -> &Card ;

    fn selected_card(&self) -> &Card;

    fn selected_id(&self) -> CardID;

    fn select_right(&mut self);

    fn select_left(&mut self);

    fn select_up(&mut self);

    fn select_down(&mut self);
}

