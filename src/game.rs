/*!

This module defines a game of colour whist, which consists of tricks and the current table

# Example Usage
You can play a whister game like this:
```no_run
use whister::game::Game;

let mut game = Game::new();
game.add_human_players(1).unwrap();

// example: three deals
let mut count = 3;
loop {
    if count == 0 {
        break;
    }
    count -= 1;

    // None supplied for just a simple rule-based opponent
    game.play_deal(&None);

    // start a new deal
    game.new_deal();
}
```

# Playing against a trained AI model
For this, you need to use a `.bin` file, a serialized fortify::Q object.
These are supplied on the github in the `data/` directory,
or can be trained using the `fortify` module.

for example:
```no_run
use whister::game::Game;
use whister::fortify;

let mut game = Game::new();
game.add_human_players(1).unwrap();

// the model is deserialized here (example file path)
let q = fortify::data::bin_to_q("~/.local/share/whister/easy.bin", false);

// example: three deals
let mut count = 3;
loop {
    if count == 0 {
        break;
    }
    count -= 1;

    // None supplied for just a simple rule-based opponent
    game.play_deal(&q);

    // start a new deal
    game.new_deal();
}
```
*/

use crate::{
    card::Card,
    deck::{CardID, Deck},
    fortify::{self, GameSpace, Q},
    gamestate::{Action, BidState, GameState},
    player::Player,
    show,
    suit::Suit,
};
use itertools::Itertools;
use std::{
    cmp::Ordering,
    io::{stdin, stdout, Write},
    process::exit,
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
    last_last_winner: usize,
    dealer: usize,
    bidding: bool,
    nb_cant_follow: [u8; 4],
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
            last_last_winner: 0,
            dealer: 0,
            bidding: true,
            nb_cant_follow: [0; 4],
        }
    }

    pub fn add_human_players(&mut self, amount: usize) -> Result<usize, String> {
        if self.human_players + amount > 4 {
            return Err("Cannot have more than 4 players to this game...".to_string());
        }

        self.human_players += amount;
        Ok(self.human_players)
    }

    pub fn new_deal(&mut self) {
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
            self.last_last_winner = self.last_winner;
            self.last_winner = self.turn;

            // add 1 to the winner's score
            self.round_scores[self.turn] += 1;

            let new_trick = self.table.pull_cards(4);

            let mut first_suit: Option<Suit> = None;

            // keep track of cards that have been seen on the table (card counting)
            // & keep track of how many players can't follow in some type
            new_trick.iter().for_each(|card| {
                if let Some(fs) = first_suit {
                    // "following" cards (just not the first time)
                    if card.suit != fs {
                        self.nb_cant_follow[fs as usize] += 1;
                    }
                } else {
                    first_suit = Some(card.suit);
                    self.nb_cant_follow[card.suit as usize] = 0;
                }
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
            self.turn = (self.turn + 1) % 4;

            Ok(())
        } else {
            Err("There are already four cards on the table. Can't play any more.".to_string())
        }
    }

    pub fn show_table(&self) {
        show::show_table(&self.table, 0, self.last_winner);
    }

    pub fn show_table_wait(&self) {
        show::show_table_wait(&self.table, 0, self.last_winner);
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
        show::clear();

        Self::welcome();
        println!();
        println!("Instructions:");
        println!("- there are 4 players.");
        println!("- each player plays one card each \"trick\"");
        println!("- the player that played the highest card wins that trick");
        println!("- the first card's suit must be \"followed\" if possible");
        println!("- if not possible (you don't have that suit), you may use any card");
        println!(
            "- hearts {} is the trump, which means that they win from any other suit",
            Suit::Hearts
        );
        println!("- if you can't follow, and don't use a trump, that card is considered lower");
        println!();
        println!("Summarized:");
        println!("- Ace > King > ... > 2");
        println!(
            "- hearts {} > {{{}, {}, {}}}",
            Suit::Hearts,
            Suit::Clubs,
            Suit::Diamonds,
            Suit::Spades
        );
        println!();

        show::wait_q();
    }

    fn input_instructions(&self) {
        println!("Press the [arrow keys] (or vim keys) to select a card.");
        println!("Press [enter] or [space bar] to play that card.");
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
                    Key::Char(' ') | Key::Char('\n') => break,
                    Key::Char('q') => {
                        stdout.flush().unwrap();
                        write!(stdout, "{}", termion::cursor::Show).unwrap();
                        drop(stdout);
                        exit(0);
                    }
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
        show::show_table(&self.table, self.turn, self.last_winner);
        show::show_last_non_empty(&self.tricks, self.last_last_winner, self.last_winner);
        println!("Your hand: [Player {}]", self.turn);
        self.players[self.turn].show_cards();
    }

    fn alowed_or_why_not(&self, card: CardID) -> Result<(), String> {
        let player = &self.players[self.turn];

        if self.table.size() != 0 && player.can_follow(self.table.card(0).suit) {
            let first_suit = self.table.card(0).suit;
            if player.card(card).suit != first_suit {
                return Err(format!(
                    "You are able to follow in {}, but are trying not to",
                    first_suit
                ));
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    fn human_plays(&mut self) {
        // showing what the game looks like atm
        self.show_player_state();
        show::wait();

        // ask what card to play and check validity
        let card_id: CardID = {
            loop {
                // loop until correct card given
                self.ask_card();

                let i = self.players[self.turn].selected_id();

                match self.alowed_or_why_not(i) {
                    Ok(_) => break i,
                    Err(err) => println!("{}", err),
                }
            }
        };

        self.player_plays(card_id)
            .expect("human player should be alowed to play selected card");
    }

    pub fn best_card_id_ai(&self, q: &Q<GameState>) -> usize {
        let best_action = {
            let best = fortify::best_action_score(q, &self.state());
            if let Ok(best) = best {
                best.0
            } else {
                <Self as fortify::GameSpace<GameState>>::random_action(self)
            }
        };

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

    fn bidding(&mut self) {
        show::dealer(self.dealer);

        // TODO: implement bidding
    }

    fn play_rounds(&mut self, q: &Option<Q<GameState>>) {
        for _ in 0..13 {
            // play one round
            for _ in 0..4 {
                if self.turn < self.human_players {
                    self.human_plays();
                    self.show_table_wait();
                } else {
                    if let Some(q) = q {
                        self.ai_plays(q);
                    } else {
                        self.rulebased_plays();
                    }
                    self.show_table_wait();
                }
            }

            self.trick().expect("Couldn't play trick in play_round");

            show::winner(self.turn);
            show::wait();
        }
    }

    pub fn play_deal(&mut self, q: &Option<Q<GameState>>) {
        // bidding
        self.bidding();

        // play the actual rounds
        self.play_rounds(q);
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
            self.new_deal();
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

    pub fn get_ai_score(&self) -> u32 {
        self.scores[0]
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

    /// returns the ID of the card that corresponds with this action
    /// note: this only returns playable card IDs!
    pub fn action_card_id(&self, action: &Action) -> CardID {
        let player = self.turn;

        let playable = self.alowed_cards();

        match action {
            Action::PlayWorst(suit) => {
                let mut suit_cards = self.of_which_suit(player, &playable, *suit as usize);
                if suit_cards.is_empty() {
                    suit_cards = playable.clone();
                }
                self.lowest_card_of(player, &suit_cards)
                    .unwrap_or(playable[0])
            }
            Action::RaiseLow => {
                let better = self.better_cards_of(player, &playable);
                self.lowest_card_of(player, &better)
                    .unwrap_or_else(|| playable[0])
            }
            Action::RaiseHigh => {
                let better = self.better_cards_of(player, &playable);
                self.highest_card_of(player, &better)
                    .unwrap_or_else(|| playable[0])
            }
            Action::TrumpHigh => {
                let trumps = self.of_which_suit(player, &playable, 3);
                self.highest_card_of(player, &trumps)
                    .unwrap_or_else(|| playable[0])
            }
            Action::TrumpLow => {
                let trumps = self.of_which_suit(player, &playable, 3);
                self.lowest_card_of(player, &trumps)
                    .unwrap_or_else(|| playable[0])
            }
            Action::PlayBest(suit) => {
                let mut suit_cards = self.of_which_suit(player, &playable, *suit as usize);
                if suit_cards.is_empty() {
                    suit_cards = playable.clone();
                }
                self.highest_card_of(player, &suit_cards)
                    .unwrap_or_else(|| playable[0])
            }
            Action::ComeBest => {
                let state: GameState = self.state();
                let suit = state.has_highest.iter().position_max().unwrap();
                let suit_ids = self.of_which_suit(player, &playable, suit);

                self.highest_card_of(player, &suit_ids)
                    .unwrap_or_else(|| playable[0])
            }
        }
    }
}

impl GameSpace<GameState> for Game {
    fn new_space(&self) -> Box<dyn GameSpace<GameState>> {
        Box::new(Self::new())
    }

    fn reward(&self) -> f64 {
        if self.last_winner == 0 {
            1.0
        } else {
            0.0
        }
    }

    fn actions(&self) -> Vec<Action> {
        let mut alowed: Vec<Action> = Vec::with_capacity(5);
        let player = self.turn;
        let playable = self.alowed_cards();
        let better = self.better_cards_of(player, &playable);
        let state: GameState = self.state();
        let first: bool = state.first_suit == -1;
        let can_follow: bool = self.can_follow(player);

        for suit in Suit::iterator() {
            alowed.push(Action::PlayWorst(*suit))
        }

        if first && state.has_highest.iter().any(|h| *h) {
            alowed.push(Action::ComeBest);
        }

        if self.players[player].can_follow(Suit::Hearts)
            && (first || !can_follow || state.first_suit == 3)
        {
            alowed.extend([Action::TrumpHigh, Action::TrumpLow]);
        }

        if can_follow && !better.is_empty() && !first {
            alowed.extend([Action::RaiseLow, Action::RaiseHigh]);
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

        let nb_cards = 0; //self.players[player].size();

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

        // compress the four 8bit numbers to four concatenated 2 bit numbers
        // saves 30% on serialized model size!!
        let mut nb_out_of: u8 = 0b00000000;
        self.nb_cant_follow.iter().enumerate().for_each(|(i, nb)| {
            // bitwise or with shifted nb
            nb_out_of |= *nb << (2 * i);
        });

        GameState {
            can_follow,
            has_highest,
            first_suit,
            have_higher,
            have_trump,
            nb_cards,
            nb_out_of,
        }
    }

    fn take_action(&mut self, action: &Action, q: &Option<&Q<GameState>>) {
        let card_id = self.action_card_id(action);
        self.agent_plays_round(card_id, q);
    }
}

/// TODO: implement this boy
impl GameSpace<BidState> for Game {
    fn reward(&self) -> f64 {
        // only give a reward after the full bidding process
        if self.bidding {
            0.0
        } else {
            // TODO: play the game to see whether or not this AI won
            todo!()
        }
    }

    fn actions(&self) -> Vec<<BidState as fortify::State>::A> {
        unimplemented!()
    }

    fn state(&self) -> BidState {
        unimplemented!()
    }

    fn take_action(
        &mut self,
        _action: &<BidState as fortify::State>::A,
        _q: &Option<&Q<BidState>>,
    ) {
        unimplemented!()
    }

    fn new_space(&self) -> Box<dyn GameSpace<BidState>> {
        unimplemented!()
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
