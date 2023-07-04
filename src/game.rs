/*!
 * This module defines a game of colour whist, which consists of tricks and the current table
 */
use crate::{
    card::Card,
    deck::{CardID, Deck},
    fortify::{GameState, QLearner},
    player::Player,
    show,
    suit::Suit,
};
use itertools::Itertools;
use std::{
    cmp::Ordering,
    io::{stdin, stdout, Write},
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
            let new_trick = self.table.pull_cards(4);

            new_trick.iter().for_each(|card| {
                self.gone_cards[card.suit as usize][(card.score() - 2) as usize] = true;
            });

            self.tricks.push(new_trick);
            Ok(())
        } else {
            Err("There are not exactly four cards on the table.".to_string())
        }
    }

    /// Put the card on the table.
    /// Comsumes the card!
    fn play(&mut self, card: Card) -> Result<(), String> {
        if self.table.size() < 4 {
            // add the card to the seen pile of cards (for AI card counting)
            self.table.add(card);
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

    pub fn player_plays(&mut self, player: PlayerID, card: usize) -> Result<(), String> {
        if player != (self.turn + self.table.size()) % 4 {
            return Err("The order of playing is incorrect.".to_string());
        }
        let alowed = self.alowed_cards(player);
        if !alowed.contains(&card) {
            return Err("This player is not alowed to play this card.".to_string());
        }

        let played = self.players[player].remove(card);
        self.play(played)
    }

    /// returns the winning card index currently on the table
    fn winner(&self) -> PlayerID {
        self.table
            .iter()
            .cloned()
            .position_max_by(|card1, card2| card1.winning(card2, &self.trump))
            .unwrap()
    }

    /// returns a vector of alowed cards for this player, in this round
    pub fn alowed_cards(&self, player: PlayerID) -> Vec<usize> {
        let player = &self.players[player];

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

    // fn play_easy(&mut self, player: PlayerID) {
    //     // get alowed indeces
    //     let playable = self.alowed_cards(player);

    //     if self.table.is_empty() {
    //         return self.play_card(player, self.highest_card_of(player, &playable).unwrap_or(playable[0]));
    //     }

    //     let better_cards = self.better_cards_of(player, &playable);

    //     if !better_cards.is_empty() {
    //         return self.play_card(player, self.lowest_card_of(player, &better_cards).unwrap_or(better_cards[0]));
    //     }

    //     // play other card
    //     self.play_card(player, self.lowest_card_of(player, &playable).unwrap_or(playable[0]));
    // }

    fn input_instructions(&self) {
        println!("Press the arrow or vim keys to move the selected card.");
        println!("Press space to enter that card.");
    }

    fn ask_card(&mut self, player: PlayerID) {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        stdout.flush().unwrap();

        let mut wrong_count = 0;
        for c in stdin.keys() {
            {
                let active_player = &mut self.players[player];

                match c.unwrap() {
                    Key::Char('h') | Key::Left => {
                        wrong_count = 0;
                        active_player.select_left()
                    }
                    Key::Char('j') | Key::Down => {
                        wrong_count = 0;
                        active_player.select_down()
                    }
                    Key::Char('k') | Key::Up => {
                        wrong_count = 0;
                        active_player.select_up()
                    }
                    Key::Char('l') | Key::Right => {
                        wrong_count = 0;
                        active_player.select_right()
                    }
                    Key::Char(' ') => break,
                    _ => {
                        wrong_count += 1;
                    }
                }
            }
            stdout.flush().unwrap();
            stdout.suspend_raw_mode().unwrap();

            if wrong_count > 0 {
                self.input_instructions();
            } else {
                self.show_player_state(player);
            }

            stdout.activate_raw_mode().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Show).unwrap();
    }

    fn show_player_state(&mut self, player: PlayerID) {
        show::show_table(&self.table);
        show::show_last_non_empty(&self.tricks);
        println!("Your hand: [Player {player}]");
        self.players[player].show_cards();
    }

    fn human_plays(&mut self, player: PlayerID) {
        // showing what the game looks like atm
        self.show_player_state(player);
        show::wait();

        // ask what card to play and check validity
        let idx: CardID = {
            let playable = self.alowed_cards(player);

            loop {
                // loop until correct card given
                self.ask_card(player);

                let i = self.players[player].selected_id();

                if playable.contains(&i) {
                    break i;
                } else {
                    println!("You can't play that card right now, try again!");
                }
            }
        };

        self.player_plays(player, idx)
            .expect("human player should be alowed to play selected card");
    }

    fn ai_plays(&mut self, player: PlayerID, learner: &mut QLearner, slow: bool) {
        let mut best_action = *learner.best_action_score(&self.state(player)).0;
        let alowed = learner.alowed_actions(self, player);

        if !alowed.iter().any(|a| *a == best_action) {
            best_action = alowed[0];
        }

        let best_card_id = learner.action_card_id(&best_action, self, player);

        self.player_plays(player, best_card_id)
            .expect("ai should be alowed to play selected card");

        if slow {
            show::show_table_wait(&self.table);
        }
    }

    /// a simple rule based AI plays a card given the current situation.
    // fn rulebased_plays(&mut self, player: PlayerID) {
    //     self.play_easy(player);

    //     show::show_table_wait(&self.table);
    // }

    pub fn play_round(&mut self, learner: &mut QLearner) {
        for i in self.turn..self.turn + 4 {
            let player = i % 4;

            if player < self.human_players {
                self.human_plays(player);

                show::show_table_wait(&self.table);
            } else {
                self.ai_plays(player, learner, true);
            }
        }

        self.turn = (self.winner() + self.turn) % 4;
        self.round_scores[self.turn] += 1;

        show::winner(self.turn);
        show::wait();

        self.trick().expect("Couldn't play trick in play_round");
    }

    pub fn agent_plays_round(&mut self, card: CardID, learner: &mut QLearner) {
        let mut plyr = 0;

        self.player_plays(plyr, card)
            .expect("agent should be alowed to play selected card");

        loop {
            if self.table.size() == 4 {
                break;
            }

            plyr += 1;
            self.ai_plays(plyr, learner, false);
        }

        self.turn = (self.winner() + self.turn) % 4;
        self.round_scores[self.turn] += 1;
        self.trick().expect("Couldn't play trick in play_round");

        if self.tricks.len() == 13 {
            self.new_round();
        }

        plyr = self.turn;
        loop {
            if plyr % 4 == 0 {
                break;
            }

            self.ai_plays(plyr, learner, false);
            plyr += 1;
        }
    }

    pub fn agent_plays_round_slowly(&mut self, card: CardID, learner: &mut QLearner) {
        self.players[0].set_selected(card);

        let mut plyr = 0;

        println!(
            "agent played card {}",
            self.players.get(0).unwrap().card(card)
        );
        self.player_plays(plyr, card)
            .expect("agent should be alowed to play selected card");

        loop {
            if self.table.size() == 4 {
                break;
            }

            plyr += 1;
            self.ai_plays(plyr, learner, true);
        }

        self.turn = (self.winner() + self.turn) % 4;
        self.round_scores[self.turn] += 1;
        self.trick()
            .expect("should be able to play trick with four cards");

        if self.tricks.len() == 13 {
            self.new_round();
        }

        plyr = self.turn;
        loop {
            if plyr % 4 == 0 {
                break;
            }

            self.ai_plays(plyr, learner, true);
            plyr += 1;
        }

        // self.show_player_state(0);
    }

    pub fn show_scores(&self) {
        println!("The scores: {:?}", self.scores);
    }

    pub fn reward(&self) -> f64 {
        if self.turn == 0 {
            1.0
        } else {
            0.0
        }
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

    pub fn state(&self, player: PlayerID) -> GameState {
        let can_follow: bool = self.can_follow(player);

        let mut has_highest = [true; 4];
        let mut first_suit = -1;
        let mut have_higher = true;
        let have_trump = self.players[player].can_follow(Suit::Hearts);

        if !self.first() {
            let first_card_suit = self.table.card(0).suit;
            first_suit = first_card_suit as i8;

            // determine whether I can go higher than the current winner
            let playable = self.alowed_cards(player);

            let winner = self.winner();

            have_higher = playable
                .iter()
                .any(|card_id| *self.players[player].card(*card_id) > self.table.cards[winner]);
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

        let result = game.player_plays(0, 0);

        assert!(result.is_ok());
    }

    #[test]
    fn player_plays_not_alowed_err() {
        let mut game = init_game();

        game.player_plays(0, 0)
            .expect("testing player should be alowed to play selected card");
        let alowed = game.alowed_cards(1);
        let mut card_id = 0;
        let not_alowed = loop {
            if !alowed.contains(&card_id) {
                break card_id;
            }
            card_id += 1;
        };
        let result = game.player_plays(1, not_alowed);

        assert!(result.is_err());
    }

    #[test]
    fn trick_four_cards_no_err() {
        let mut game = init_game();

        game.player_plays(0, 0)
            .expect("testing player should be alowed to play selected card");
        for plyr in 1..4 {
            let alowed = game.alowed_cards(plyr);
            game.player_plays(plyr, alowed[0])
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
        for plyr in 1..4 {
            if game.players[plyr].can_follow(Suit::Clubs) {
                assert!(game
                    .alowed_cards(plyr)
                    .iter()
                    .all(|card_id| game.players[plyr].card(*card_id).suit == Suit::Clubs));
            }
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
            let playable = &game.alowed_cards(plyr);
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
