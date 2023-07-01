/*!
 * This module defines a game of colour whist, which consists of tricks and the current table
 */
use crate::{
    card::Card,
    deck::{CardID, Deck},
    fortify::GameState,
    player::Player,
    show,
    suit::Suit,
};

use device_query::{DeviceQuery, DeviceState, Keycode};

type PlayerID = usize;

#[derive(Hash, Eq, PartialEq)]
pub struct Game {
    /// tricks keeps track of the played tricks
    /// one trick is a deck of 4 cards
    tricks: Vec<Deck>,
    /// table keeps track of which cards are
    /// on the table already
    table: Deck,
    pub players: [Player; 4],
    turn: PlayerID,
    trump: Suit,
    scores: [u32; 4],
    human_players: usize,
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
            human_players: 0
        }
    }

    pub fn add_human_players(&mut self, amount: usize) -> Result<usize, String> {
        if self.human_players + amount > 4 {
            return Err("Cannot have more than 4 players to this game...".to_string())
        }
        
        self.human_players += amount;
        Ok(self.human_players)
    }

    pub fn new_round(&mut self) {
        let mut deck = Deck::new_full();
        deck.shuffle();

        let cards = deck.pull_cards(13);
        self.players[0].cards = cards;

        let cards = deck.pull_cards(13);
        self.players[1].cards = cards;

        let cards = deck.pull_cards(13);
        self.players[2].cards = cards;

        let cards = deck.pull_cards(13);
        self.players[3].cards = cards;

        // self.turn = 0;
    }

    pub fn trick(&mut self) -> Result<(), String> {
        if self.table.size() == 4 {
            let new_trick = self.table.pull_cards(4);
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
            self.table.add(card);
            Ok(())
        } else {
            Err(String::from(
                "There are already four cards on the table. Can't play any more.",
            ))
        }
    }

    pub fn show_table(&self) {
        self.table.show();
    }

    pub fn play_card(&mut self, player: PlayerID, card: usize) {
        let played = self.players[player].cards.remove(card);
        self.play(played).expect("Couldn't play card");
    }

    /// returns the winning card index currently on the table
    fn winner(&self) -> PlayerID {
        let mut best_card = self.table.card(0);
        let mut winner = 0;

        for i in 1..self.table.size() {
            let new_card = self.table.card(i);

            if new_card.better(best_card, &self.trump) {
                best_card = new_card;
                winner = i;
            }
        }

        winner
    }

    /// returns a vector of alowed cards for this player, in this round
    fn alowed_cards(&self, player: PlayerID) -> Vec<usize> {
        // if the table is empty, every card is alowed
        let mut result: Vec<CardID> = Vec::new();

        let player = &self.players[player];

        if self.table.size() != 0 {
            // i'm possibly restricted to the first-layed card this trick
            let first_suit = self.table.card(0).suit;

            if player.cards.has_suit(&first_suit) {
                // only return indices of cards of said suit
                for i in 0..player.cards.size() {
                    if player.cards.suit_of(i) == first_suit {
                        result.push(i);
                    }
                }
                return result;
            }
        }

        // if I got here, i am free to play any card
        for i in 0..player.cards.size() {
            result.push(i);
        }

        result
    }

    fn better_cards_of(&self, player: PlayerID, playable: &[CardID]) -> Vec<CardID> {
        if self.table.size() == 0 {
            return playable.to_vec();
        }

        let player = &self.players[player];
        let best_on_table = self.table.card(self.winner());

        playable
            .iter()
            .cloned()
            .filter(|card| player.card(*card).better(best_on_table, &self.trump))
            .collect()
    }

    fn highest_card_of(&self, player: PlayerID, out_of: &[CardID]) -> CardID {
        self.players[player].cards.highest(out_of, &Suit::Hearts)
    }

    fn lowest_card_of(&self, player: PlayerID, out_of: &[CardID]) -> CardID {
        self.players[player].cards.lowest(out_of, &Suit::Hearts)
    }

    fn play_easy(&mut self, player: PlayerID) {
        // get alowed indeces
        let playable = self.alowed_cards(player);

        if self.table.size() == 0 {
            return self.play_card(player, self.highest_card_of(player, &playable));
        }

        let better_cards = self.better_cards_of(player, &playable);

        if !better_cards.is_empty() {
            return self.play_card(player, self.lowest_card_of(player, &better_cards));
        }

        // play other card
        self.play_card(player, self.lowest_card_of(player, &playable));
    }

    fn ask_card(&mut self, player: PlayerID) { 
        let device_state = DeviceState::new();

        let mut prev_keys: Vec<Keycode> = Vec::new();
        let mut keys: Vec<Keycode>;

        print!("\x1b[?25l");

        loop {
            keys = device_state.get_keys();

            if keys == prev_keys || keys.is_empty() {
                prev_keys = keys;
                continue
            }

            println!("{:?}", keys);

            {    
                let active_player = &mut self.players[player];

                match keys[keys.len()-1] {
                    Keycode::H | Keycode::Left => active_player.cards.select_left(),
                    Keycode::J | Keycode::Down => active_player.cards.select_down(),
                    Keycode::K | Keycode::Up => active_player.cards.select_up(),
                    Keycode::L | Keycode::Right => active_player.cards.select_right(),
                    Keycode::Enter | Keycode::Space => break,
                    _ => (),
                }
            }

            prev_keys = keys;

            self.show_player_state(player);
        }
        print!("\x1b[?25h");
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

        self.play_card(player, idx);
    }

    pub fn play_round(&mut self) {
        for i in self.turn..self.turn + 4 {
            let player = i % 4;

            if player < self.human_players  {
                self.human_plays(player);

                show::show_table_wait(&self.table);
            } else {
                // AI thinks for a little while
                // thread::sleep(Duration::from_millis(500));

                self.play_easy(player);

                show::show_table_wait(&self.table);
            }
        }

        self.turn = (self.winner() + self.turn) % 4;
        self.scores[self.turn] += 1;

        show::winner(self.turn);

        self.trick().expect("Couldn't play trick in play_round");
    }

    pub fn show_scores(&self) {
        println!("The scores: {:?}", self.scores);
    }

    pub fn reward(&self) -> u32 {
        self.scores[0]
    }

    pub fn state(&self) -> GameState {
        //TODO: implement me
        GameState::new()
    }
}
