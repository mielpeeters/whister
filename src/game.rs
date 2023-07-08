//!
//! This module defines a game of colour whist, which consists of tricks and the current table
//! 
//! # Example Usage
//! You can play a whister game like this:
//! ```no_run
//! # use whister::game::Game;
//! #
//! let mut game = Game::new();
//! game.add_human_players(1).unwrap();
//! 
//! // example: three rounds
//! let mut count = 3;
//! loop {
//!     if count == 0 {
//!         break;
//!     }
//!     count -= 1;
//!     
//!     for _ in 0..13 {
//!         // None supplied for just a simple rule-based opponent
//!         game.play_round(&None);
//!     }
//!     
//!     // start a new round
//!     game.new_round();
//! }
//! ```
//! 
//! # Playing against a trained AI model
//! For this, you need to use a `.pkl` file, a serialized fortify::Q object.
//! These are supplied on the github in the `data/` directory, 
//! or can be trained using the `fortify` module.
//! 
//! for example:
//! ```no_run
//! # use whister::game::Game;
//! use whister::fortify;
//! let mut game = Game::new();
//! game.add_human_players(1).unwrap();
//! 
//! // the model is deserialized here
//! let q =  fortify::pickle_to_q("data/whister.pkl", false);
//! 
//! // example: three rounds
//! let mut count = 3;
//! loop {
//!     if count == 0 {
//!         break;
//!     }
//!     count -= 1;
//!     
//!     for _ in 0..13 {
//!         // None supplied for just a simple rule-based opponent
//!         game.play_round(&q);
//!     }
//!     
//!     // start a new round
//!     game.new_round();
//! }
//! ```

use crate::{
    card::Card,
    deck::{CardID, Deck},
    fortify::{self, GameSpace, Q},
    gamestate::{Action, GameState},
    player::Player,
    show,
    suit::Suit,
};
use itertools::Itertools;
use std::{
    cmp::Ordering,
    io::{stdin, stdout, Write}, process::exit,
};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

type PlayerID = usize;

#[derive(Hash, Eq, PartialEq)]
pub struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    tricks: Vec<Deck>,
    /// table keeps track of which cards are
    /// on the table already
    table: Deck,
    pub players: [Deck; 4],
    turn: PlayerID,
    trump: Suit,
    scores: [u32; 4],
    human_players: usize,
    round_scores: [u32; 4],
    gone_cards: [[bool; 13]; 4],
    last_winner: usize,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        let mut deck = Deck::new_full();
        deck.shuffle();
        let tricks: Vec<Deck> = Vec::new();
        let table: Deck = Deck::new_empty();

        // create new players
        let player_one = Player::new_take_cards(&mut deck, 13);
        let player_two = Player::new_take_cards(&mut deck, 13);
        let player_three = Player::new_take_cards(&mut deck, 13);
        let player_four = Player::new_take_cards(&mut deck, 13);

        let players = [player_one, player_two, player_three, player_four];
        let scores = [0, 0, 0, 0];

        Game {
            tricks,
            table,
            players,
            turn: 0,
            trump: Suit::Hearts,
            scores,
            human_players: 0,
            round_scores: [0; 4],
            gone_cards: [[false; 13]; 4],
            last_winner: 0,
        }
    }

    pub fn add_human_players(&mut self, amount: usize) -> Result<usize, String> {
        if self.human_players + amount > 4 {
            return Err("Cannot have more than 4 players to this game...".to_string());
        }

        self.human_players += amount;
        Ok(self.human_players)
    }

    pub fn new_round(&mut self) {
        let mut deck = Deck::new_full();
        deck.shuffle();

        let tricks: Vec<Deck> = Vec::new();
        self.tricks = tricks;

        self.gone_cards = [[false; 13]; 4];

        self.round_scores
            .iter()
            .enumerate()
            .for_each(|(plyr, score)| {
                self.scores[plyr] += score;
            });

        self.round_scores = [0; 4];

        let cards = deck.pull_cards(13);
        self.players[0] = cards;
        self.players[0].sort();

        let cards = deck.pull_cards(13);
        self.players[1] = cards;
        self.players[1].sort();

        let cards = deck.pull_cards(13);
        self.players[2] = cards;
        self.players[2].sort();

        let cards = deck.pull_cards(13);
        self.players[3] = cards;
        self.players[3].sort();
    }

    pub fn trick(&mut self) -> Result<(), String> {
        if self.table.size() == 4 {
            // determine winning player, set turn to them
            self.turn = (self.winner() + self.last_winner) % 4;
            self.last_winner = self.turn;

            // add 1 to the winner's score
            self.round_scores[self.turn] += 1;

            let new_trick = self.table.pull_cards(4);

            // keep track of cards that have been seen on the table (card counting)
            new_trick.iter().for_each(|card| {
                self.gone_cards[card.suit as usize][(card.score() - 2) as usize] = true;
            });

            self.tricks.push(new_trick);

            Ok(())
        } else {
            Err("There are not exactly four cards on the table.".to_string())
        }
    }

    // fn current_player(&self) -> &Deck {
    //     &self.players[self.turn]
    // }

    /// Put the card on the table.
    /// Comsumes the card!
    fn play(&mut self, card: Card) -> Result<(), String> {
        if self.table.size() < 4 {
            // add the card to the seen pile of cards (for AI card counting)
            self.table.add(card);
            self.turn = (self.turn + 1) % 4;

            Ok(())
        } else {
            Err("There are already four cards on the table. Can't play any more.".to_string())
        }
    }

    pub fn show_table(&self) {
        self.table.show();
    }

    pub fn show_table_wait(&self) {
        show::show_table_wait(&self.table);
    }

    pub fn player_plays(&mut self, card: usize) -> Result<(), String> {
        let alowed = self.alowed_cards();
        if !alowed.contains(&card) {
            return Err("This player is not alowed to play this card.".to_string());
        }

        let played = self.players[self.turn].remove(card);
        self.play(played)
    }

    /// returns the winning card index currently on the table
    fn winner(&self) -> CardID {
        self.table
            .iter()
            .cloned()
            .position_max_by(|card1, card2| card1.winning(card2, &self.trump))
            .unwrap()
    }

    /// returns a vector of alowed cards for this player, in this round
    pub fn alowed_cards(&self) -> Vec<usize> {
        let player = &self.players[self.turn];

        if self.table.size() != 0 && player.can_follow(self.table.card(0).suit) {
            let first_suit = self.table.card(0).suit;
            // only return indices of cards of said suit
            return player
                .iter()
                .enumerate()
                .filter(|(_, card)| card.suit == first_suit)
                .map(|(i, _)| i)
                .collect_vec();
        }

        (0..player.size()).collect_vec()
    }

    pub fn card_id_of(&self, player: PlayerID, card: &Card) -> Option<CardID> {
        self.players[player].id_of(card)
    }

    pub fn better_cards_of(&self, player: PlayerID, playable: &[CardID]) -> Vec<CardID> {
        if self.table.is_empty() {
            return playable.to_vec();
        }

        let player = &self.players[player];
        let best_on_table = self.table.card(self.winner());

        playable
            .iter()
            .cloned()
            .filter(|card| {
                player.card(*card).winning(best_on_table, &self.trump) == Ordering::Greater
            })
            .collect()
    }

    pub fn highest_card_of(&self, player: PlayerID, out_of: &[CardID]) -> Option<CardID> {
        self.players[player].highest(out_of, &Suit::Hearts)
    }

    pub fn lowest_card_of(&self, player: PlayerID, out_of: &[CardID]) -> Option<CardID> {
        self.players[player].lowest(out_of, &Suit::Hearts)
    }

    pub fn of_which_suit(&self, player: PlayerID, out_of: &[CardID], suit: usize) -> Vec<CardID> {
        let player = self.players.get(player).unwrap();

        out_of
            .iter()
            .cloned()
            .filter(|card| player.card(*card).suit as usize == suit)
            .collect()
    }

    fn play_easy(&mut self) {
        // get player id from the current turn
        let player = self.turn;

        // get alowed indeces
        let playable = self.alowed_cards();

        if self.table.is_empty() {
            return self
                .player_plays(
                    self.highest_card_of(player, &playable)
                        .unwrap_or(playable[0]),
                )
                .expect("test");
        }

        let better_cards = self.better_cards_of(player, &playable);

        if !better_cards.is_empty() {
            return self
                .player_plays(
                    self.lowest_card_of(player, &better_cards)
                        .unwrap_or(better_cards[0]),
                )
                .expect("test");
        }

        // play other card
        self.player_plays(
            self.lowest_card_of(player, &playable)
                .unwrap_or(playable[0]),
        )
        .expect("test");
    }

    fn print_boxed(input: &str, tab: usize) {
        let width = input.len();
        println!("{}╭{}╮", " ".repeat(tab), "─".repeat(width));
        println!("{}│\x1b[1m{}\x1b[0m│", " ".repeat(tab), input);
        println!("{}╰{}╯", " ".repeat(tab), "─".repeat(width));
    }

    fn welcome() {
        Self::print_boxed("Welcome to Whister", 10);
    }

    pub fn instructions() {
        Self::welcome();
        println!();

        println!("Instructions:");
        println!("- there are 4 players.");
        println!("- each player plays one card each \"trick\"");
        println!("- the player that played the highest card wins that trick");
        println!("- the first card's suit must be \"followed\" if possible");
        println!("- if not possible (you don't have that suit), you may use any card");
        println!("- hearts ♥ is the trump, which means that they win from any other suit");
        println!("- if you can't follow, and don't use a trump, that card is considered lower");

        println!("Summarized:");
        println!("- Ace > King > ... > 2");
        println!("- hearts ♥ > {{others}}");

        Self::wait_q();
    }

    fn input_instructions(&self) {
        println!("Press the arrow or vim keys to move the selected card.");
        println!("Press space to enter that card.");
    }

    fn wait_q() {
        println!("Press q to continue.");

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        stdout.flush().unwrap();

        for c in stdin.keys() {
            {
                if let Key::Char('q') =  c.unwrap() {
                    break;
                }
            }
        }
        write!(stdout, "{}", termion::cursor::Show).unwrap();
    }

    fn ask_card(&mut self) {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        stdout.flush().unwrap();

        let mut wrong_count = 0;
        for c in stdin.keys() {
            {
                match c.unwrap() {
                    Key::Char('h') | Key::Left => {
                        wrong_count = 0;
                        self.players[self.turn].select_left()
                    }
                    Key::Char('j') | Key::Down => {
                        wrong_count = 0;
                        self.players[self.turn].select_down()
                    }
                    Key::Char('k') | Key::Up => {
                        wrong_count = 0;
                        self.players[self.turn].select_up()
                    }
                    Key::Char('l') | Key::Right => {
                        wrong_count = 0;
                        self.players[self.turn].select_right()
                    }
                    Key::Char(' ') => break,
                    Key::Char('q') => {
                        exit(0)
                    },
                    _ => {
                        wrong_count += 1;
                    }
                }
            }
            stdout.flush().unwrap();
            stdout.suspend_raw_mode().unwrap();

            if wrong_count > 1 {
                self.input_instructions();
            } else {
                self.show_player_state();
            }

            stdout.activate_raw_mode().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Show).unwrap();
    }

    pub fn show_player_state(&mut self) {
        show::show_table(&self.table);
        show::show_last_non_empty(&self.tricks);
        println!("Your hand: [Player {}]", self.turn);
        self.players[self.turn].show_cards();
    }

    fn human_plays(&mut self) {
        // showing what the game looks like atm
        self.show_player_state();
        show::wait();

        // ask what card to play and check validity
        let card_id: CardID = {
            let playable = self.alowed_cards();

            loop {
                // loop until correct card given
                self.ask_card();

                let i = self.players[self.turn].selected_id();

                if playable.contains(&i) {
                    break i;
                } else {
                    println!("You can't play that card right now, try again!");
                }
            }
        };

        self.player_plays(card_id)
            .expect("human player should be alowed to play selected card");
    }

    pub fn best_card_id_ai(&self, q: &Q<GameState>) -> usize {
        let mut best_action = {
            let best = fortify::best_action_score(q, &self.state());
            if let Ok(best) = best {
                best.0
            } else {
                self.random_action()
            }
        };

        let alowed = self.actions();

        if !alowed.iter().any(|a| *a == best_action) {
            best_action = alowed[0];
        }

        self.action_card_id(&best_action)
    }

    fn ai_plays(&mut self, q: &Q<GameState>) {
        let best_card_id = self.best_card_id_ai(q);

        self.player_plays(best_card_id)
            .expect("ai should be alowed to play selected card");
    }

    /// a simple rule based AI plays a card given the current situation.
    fn rulebased_plays(&mut self) {
        self.play_easy();
    }

    pub fn play_round(&mut self, q: &Option<Q<GameState>>) {
        for _ in 0..4 {
            if self.turn < self.human_players {
                self.human_plays();
                show::show_table_wait(&self.table);
            } else {
                if let Some(q) = q {
                    self.ai_plays(q);
                } else {
                    self.rulebased_plays();
                }
                show::show_table_wait(&self.table);
            }
        }

        self.trick().expect("Couldn't play trick in play_round");

        show::winner(self.turn);
        show::wait();
    }

    pub fn agent_plays_round(&mut self, card: CardID, q: &Option<&Q<GameState>>) {
        self.player_plays(card)
            .expect("agent should be alowed to play selected card");

        // let opponent play until the current trick is full
        loop {
            if self.table.size() == 4 {
                break;
            }

            if let Some(q) = q {
                self.ai_plays(q);
            } else {
                self.rulebased_plays();
            }
        }

        // complete the trick
        self.trick()
            .expect("Should finish trick in agent_plays_round");

        // start a new round if necessary
        if self.tricks.len() == 13 {
            self.new_round();
        }

        // let first opponents put their cards down, until player 0 is up
        loop {
            if self.turn == 0 {
                break;
            }

            if let Some(q) = q {
                self.ai_plays(q);
            } else {
                self.rulebased_plays();
            }
        }
    }

    pub fn show_scores(&self) {
        println!("The scores: {:?}", self.scores);
    }

    pub fn can_follow(&self, player: PlayerID) -> bool {
        if self.table.is_empty() {
            return true;
        }

        if self.players[player].can_follow(self.table.card(0).suit) {
            return true;
        }

        false
    }

    pub fn first(&self) -> bool {
        self.table.is_empty()
    }

    pub fn show_gone(&self) {
        println!("Gone Cards:\n{:?}", self.gone_cards);
    }

    pub fn action_card_id(&self, action: &Action) -> CardID {
        let player = self.turn;

        let playable = self.alowed_cards();

        match action {
            Action::PlayWorst => self
                .lowest_card_of(player, &playable)
                .unwrap_or(playable[0]),
            Action::RaiseLow => {
                let better = self.better_cards_of(player, &playable);
                self.lowest_card_of(player, &better).unwrap_or(playable[0])
            }
            Action::RaiseHigh => {
                let better = self.better_cards_of(player, &playable);
                self.highest_card_of(player, &better).unwrap_or(playable[0])
            }
            Action::TrumpHigh => {
                let trumps = self.of_which_suit(player, &playable, 3);
                self.highest_card_of(player, &trumps).unwrap_or(playable[0])
            }
            Action::TrumpLow => {
                let trumps = self.of_which_suit(player, &playable, 3);
                self.lowest_card_of(player, &trumps).unwrap_or(playable[0])
            }
            Action::PlayBest => self
                .highest_card_of(player, &playable)
                .unwrap_or(playable[0]),
            Action::ComeBest => {
                let state = self.state();
                let suit = state.has_highest.iter().position_max().unwrap();
                let suit_ids = self.of_which_suit(player, &playable, suit);

                self.highest_card_of(player, &suit_ids)
                    .unwrap_or(playable[0])
            }
        }
    }
}

impl GameSpace<GameState> for Game {
    fn reward(&self) -> f64 {
        if self.last_winner == 0 {
            1.0
        } else {
            0.0
        }
    }

    fn actions(&self) -> Vec<Action> {
        let mut alowed: Vec<Action> = Vec::new();

        let player = self.turn;

        let playable = self.alowed_cards();
        let better = self.better_cards_of(player, &playable);

        let state = self.state();
        let first: bool = state.first_suit == -1;

        alowed.push(Action::PlayWorst);

        if first && state.has_highest.iter().any(|h| *h) {
            alowed.push(Action::ComeBest);
        }

        if self.players[player].can_follow(Suit::Hearts)
            && (first || !self.can_follow(player) || state.first_suit == 3)
        {
            alowed.push(Action::TrumpHigh);
            alowed.push(Action::TrumpLow);
        }

        if self.can_follow(player) && !better.is_empty() && !first {
            alowed.push(Action::RaiseLow);
            alowed.push(Action::RaiseHigh);
        }

        alowed
    }

    fn state(&self) -> GameState {
        let player = self.turn;

        let can_follow: bool = self.can_follow(player);

        let mut has_highest = [true; 4];
        let mut first_suit = -1;
        let mut have_higher = true;
        let have_trump = self.players[player].can_follow(Suit::Hearts);

        if !self.first() {
            let first_card_suit = self.table.card(0).suit;
            first_suit = first_card_suit as i8;

            // determine whether I can go higher than the current winner
            let playable = self.alowed_cards();

            let winner = self.winner();

            have_higher = playable.iter().any(|card_id| {
                self.players[player]
                    .card(*card_id)
                    .winning(&self.table.cards[winner], &self.trump)
                    == Ordering::Greater
            });
        }

        for s in Suit::iterator() {
            let ai_suit_deck = self.players[player].get_deck_of_suit(s);

            if let Some(my_highest) = ai_suit_deck.iter().max() {
                for i in (my_highest.score() + 1)..15 {
                    if !self.gone_cards[*s as usize][(i - 2) as usize] {
                        has_highest[*s as usize] = false;
                        break;
                    }
                }
            } else {
                has_highest[*s as usize] = false;
            }
        }

        GameState {
            can_follow,
            has_highest,
            first_suit,
            have_higher,
            have_trump,
        }
    }

    fn take_action(&mut self, action: &Action, q: &Option<&Q<GameState>>) {
        let card_id = self.action_card_id(action);
        self.agent_plays_round(card_id, q);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_game() -> Game {
        Game::new()
    }

    #[test]
    fn trick_no_cards_err() {
        let mut game = init_game();

        let result = game.trick();

        assert!(result.is_err(), "No error when tricking with no cards.");
    }

    #[test]
    fn player_plays_alowed_no_err() {
        let mut game = init_game();

        let result = game.player_plays(0);

        assert!(result.is_ok());
    }

    #[test]
    fn player_plays_not_alowed_err() {
        let mut game = init_game();

        game.player_plays(0)
            .expect("testing player should be alowed to play selected card");
        let alowed = game.alowed_cards();
        let mut card_id = 0;
        let not_alowed = loop {
            if !alowed.contains(&card_id) {
                break card_id;
            }
            card_id += 1;
        };
        let result = game.player_plays(not_alowed);

        assert!(result.is_err());
    }

    #[test]
    fn trick_four_cards_no_err() {
        let mut game = init_game();

        game.player_plays(0)
            .expect("testing player should be alowed to play selected card");
        for _ in 1..4 {
            let alowed = game.alowed_cards();
            game.player_plays(alowed[0])
                .expect("testing player should be alowed to play selected card");
        }

        let result = game.trick();

        assert!(result.is_ok(), "Error when tricking with four cards.");
    }

    #[test]
    fn winner_same_suit() {
        let mut game = init_game();

        game.play(Card {
            suit: Suit::Clubs,
            number: 2,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Clubs,
            number: 3,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Clubs,
            number: 4,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Clubs,
            number: 1,
        })
        .expect("test player should be alowed to play");

        let winner = game.winner();

        assert_eq!(winner, 3)
    }

    #[test]
    fn winner_different_suits() {
        let mut game = init_game();

        game.play(Card {
            suit: Suit::Clubs,
            number: 5,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Spades,
            number: 3,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Diamonds,
            number: 4,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Clubs,
            number: 2,
        })
        .expect("test player should be alowed to play");

        let winner = game.winner();

        assert_eq!(winner, 0)
    }

    #[test]
    fn winner_trumped_once() {
        let mut game = init_game();

        game.play(Card {
            suit: Suit::Clubs,
            number: 5,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Spades,
            number: 3,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Hearts,
            number: 2,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Diamonds,
            number: 4,
        })
        .expect("test player should be alowed to play");

        let winner = game.winner();

        assert_eq!(winner, 2)
    }

    #[test]
    fn winner_trumped_twice() {
        let mut game = init_game();

        game.play(Card {
            suit: Suit::Clubs,
            number: 5,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Spades,
            number: 3,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Hearts,
            number: 2,
        })
        .expect("test player should be alowed to play");
        game.play(Card {
            suit: Suit::Hearts,
            number: 4,
        })
        .expect("test player should be alowed to play");

        let winner = game.winner();

        assert_eq!(winner, 3)
    }

    #[test]
    fn alowed_cards() {
        let mut game = init_game();

        // fix first card
        game.play(Card {
            suit: Suit::Clubs,
            number: 3,
        })
        .expect("tet player should be alowed to play any first card");

        // test for all other players
        if game.players[game.turn].can_follow(Suit::Clubs) {
            assert!(game
                .alowed_cards()
                .iter()
                .all(|card_id| game.players[game.turn].card(*card_id).suit == Suit::Clubs));
        }
    }

    #[test]
    fn better_cards() {
        let mut game = init_game();

        let first_card = Card {
            suit: Suit::Clubs,
            number: 5,
        };

        // fix first card
        game.play(first_card.clone())
            .expect("tet player should be alowed to play any first card");

        for plyr in 1..4 {
            let playable = &game.alowed_cards();
            assert!(game
                .better_cards_of(plyr, playable)
                .iter()
                .all(|card_id| game.players[plyr]
                    .card(*card_id)
                    .winning(&first_card, &game.trump)
                    == Ordering::Greater));
        }
    }
}
