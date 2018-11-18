use deck::{Card, Deck, Suit};
use super::messages::CardColour;
use super::history::*;
use std::collections::VecDeque;

pub struct RedOrBlack {
    usernames: Vec<String>,
    index: usize,
    penalty: u16,
    deck: Deck,
    card_history: CardHistory,
    game_history: GameHistory,
    turn_number: u16,
}

impl RedOrBlack {
    pub fn new(usernames: Vec<String>) -> Self {
        RedOrBlack {
            usernames,
            index: 0,
            penalty: 5,
            deck: Deck::new_shuffled(),
            card_history: CardHistory::new(3),
            game_history: GameHistory::new(40),
            turn_number: 1,
        }
    }

    pub fn get_card_history(&self) -> &VecDeque<Option<Card>> {
        self.card_history.get_history()
    }

    pub fn get_game_history(&self) -> &Vec<HistoryItem> {
        self.game_history.get_history()
    }

    pub fn cards_left(&self) -> usize {
        self.deck.len()
    }

    pub fn get_penalty(&self) -> u16 {
        self.penalty
    }

    pub fn increment_penalty(&mut self) -> u16 {
        self.penalty += 5;
        self.penalty
    }

    pub fn reset_penalty(&mut self) -> u16 {
        self.penalty = 5;
        self.penalty
    }

    pub fn get_current_player(&mut self) -> Option<&String> {
        // Check bounds incase len has shrunk from players leaving
        if self.index >= self.usernames.len() {
            self.index = 0;
        }
        self.usernames.get(self.index)
    }

    pub fn next_player(&mut self) -> Option<&String> {
        // Check bounds incase len has shrunk from players leaving
        self.index += 1;
        if self.index >= self.usernames.len() {
            self.index = 0;
        }

        self.usernames.get(self.index)
    }

    pub fn remove_player(&mut self, username: &str) -> bool {
        let mut changed_turn = false;
        // First check if there is a current player
        if let Some(current_player) = self.get_current_player().as_ref() {
            // If the current player is the player being removed, then we need to progress the game
            // to the next player
            if current_player == &username {
                changed_turn = true;
            }
        }

        // If the turn needs to changed, then change it
        if changed_turn {
            self.next_player();
        }

        // Find posistion of player to remove
        if let Some(index) = self.usernames.iter().position(|u| u == username) {
            self.usernames.remove(index);
        }

        if self.usernames.is_empty() {
            // Reset game since we have 0 players
            // If someone joins after this it's basically a new game
            self.reset();
        }

        changed_turn
    }

    pub fn add_player(&mut self, p: String) {
        self.usernames.push(p);
    }

    pub fn draw_card(&mut self) -> Card {
        if let Some(card) = self.deck.pop() {
            card
        } else {
            info!("Deck finished!!! re-shuffling");
            self.deck = Deck::new_shuffled();
            self.deck.pop().unwrap()
        }
    }

    pub fn validate_guess(&self, guess: &CardColour, card: Card) -> bool {
        guess == &CardColour::Black && (card.suit == Suit::Spade || card.suit == Suit::Club)
            || guess == &CardColour::Red && (card.suit == Suit::Heart || card.suit == Suit::Diamond)
    }

    // validate guess, and change players turn
    // return (correct, penalty, next user, the card drawn, and number of cards left)
    pub fn play_turn(&mut self, guess: &CardColour) -> (bool, u16, Option<&String>, Card, usize) {
        let card = self.draw_card();
        self.card_history.push(card);
        let correct = self.validate_guess(guess, card);
        let penalty = if correct {
            self.increment_penalty()
        } else {
            let penalty = self.penalty;
            self.reset_penalty();
            penalty
        };

        let history_item = HistoryItem {
            username: self.get_current_player().cloned().unwrap_or("".to_string()),
            guess: guess.clone(),
            outcome: correct,
            card,
            penalty,
            turn_number: self.turn_number,
        };

        self.game_history.push(history_item);
        self.turn_number += 1;

        let cards_left = self.deck.len();
        let player = self.next_player();
        (correct, penalty, player, card, cards_left)
    }

    fn reset(&mut self) {
        info!("Reseting game");
        self.penalty = 5;
        self.card_history = CardHistory::new(3);
        self.game_history = GameHistory::new(40);
        self.deck = Deck::new_shuffled();
        self.turn_number = 1;
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    mod penalty {
        use super::*;
        use red_or_black::messages::CardColour;

        #[test]
        fn starts_at_five() {
            let usernames = vec!["mick".to_string()];
            let game = RedOrBlack::new(usernames);
            assert_eq!(game.get_penalty(), 5);
        }

        #[test]
        fn increments_by_five() {
            let usernames = vec!["mick".to_string()];
            let mut game = RedOrBlack::new(usernames);
            game.increment_penalty();
            assert_eq!(game.get_penalty(), 10);
        }

        #[test]
        fn incorrect_guess_increments() {
            let usernames = vec!["mick".to_string()];
            let mut game = RedOrBlack::new(usernames);
            let mut correct_count = 1;
            let guess = CardColour::Red;
            // while we guess correctly the penalty should not change
            while game.play_turn(&guess).0 == true {
                correct_count += 1;
                assert_eq!(game.get_penalty(), 5 * correct_count);
            }
            // After a wrong guess the penalty should be reset
            assert_eq!(game.get_penalty(), 5);
        }
    }

    mod player {
        use super::*;
        use red_or_black::messages::CardColour;

        #[test]
        fn with_zero_players() {
            let mut game = RedOrBlack::new(Vec::new());
            let guess = CardColour::Black;
            assert_eq!(game.get_current_player(), None);
            assert_eq!(game.next_player(), None);
            assert_eq!(game.play_turn(&guess).2, None);
        }

        #[test]
        fn with_one_player() {
            let mut game = RedOrBlack::new(vec!["mick".to_string()]);
            let guess = CardColour::Black;
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            assert_eq!(game.next_player(), Some(&"mick".to_string()));
            assert_eq!(game.next_player(), Some(&"mick".to_string()));
            assert_eq!(game.play_turn(&guess).2, Some(&"mick".to_string()));
            assert_eq!(game.play_turn(&guess).2, Some(&"mick".to_string()));
        }

        #[test]
        fn with_players() {
            let mut game = RedOrBlack::new(vec!["mick".to_string(), "john".to_string()]);
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));

            assert_eq!(game.next_player(), Some(&"john".to_string()));
            assert_eq!(game.get_current_player(), Some(&"john".to_string()));

            assert_eq!(game.next_player(), Some(&"mick".to_string()));
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
        }

        #[test]
        fn remove_the_only_player() {
            let mut game = RedOrBlack::new(vec!["mick".to_string()]);
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            game.remove_player("mick");
            assert_eq!(game.get_current_player(), None);
        }

        #[test]
        fn remove_one_of_two_players() {
            let mut game = RedOrBlack::new(vec!["mick".to_string(), "john".to_string()]);
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            game.remove_player("mick");
            assert_eq!(game.get_current_player(), Some(&"john".to_string()));
        }

        #[test]
        fn add_player() {
            let mut game = RedOrBlack::new(vec![]);
            assert_eq!(game.get_current_player(), None);
            assert_eq!(game.next_player(), None);

            game.add_player("mick".to_string());
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            assert_eq!(game.next_player(), Some(&"mick".to_string()));

            game.add_player("john".to_string());
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            assert_eq!(game.next_player(), Some(&"john".to_string()));

            assert_eq!(game.next_player(), Some(&"mick".to_string()));
            game.add_player("begbie".to_string());
            assert_eq!(game.get_current_player(), Some(&"mick".to_string()));
            assert_eq!(game.next_player(), Some(&"john".to_string()));
            assert_eq!(game.next_player(), Some(&"begbie".to_string()));
        }
    }

    mod card_history {
        use super::*;

        #[test]
        fn card_gets_added_to_history() {
            let mut game = RedOrBlack::new(vec!["renton".to_string()]);
            let guess = CardColour::Red;
            game.play_turn(&guess);
            let history = game.get_card_history();
            assert!(history[0].is_some());
            assert!(history[1].is_none());
            assert!(history[2].is_none());
        }

        #[test]
        fn history_doesnt_grow() {
            let mut game = RedOrBlack::new(vec!["renton".to_string()]);
            let guess = CardColour::Red;
            let (_, _, _, _, cards1) = game.play_turn(&guess);
            let (_, _, _, card2, cards2) = game.play_turn(&guess);
            let (_, _, _, card3, cards3) = game.play_turn(&guess);
            let (_, _, _, card4, cards4) = game.play_turn(&guess);
            let history = game.get_card_history();
            assert_eq!(cards1, 51);
            assert_eq!(cards2, 50);
            assert_eq!(cards3, 49);
            assert_eq!(cards4, 48);
            assert_eq!(history[0], Some(card4));
            assert_eq!(history[1], Some(card3));
            assert_eq!(history[2], Some(card2));
            assert_eq!(history.len(), 3);
        }
    }

    #[test]
    fn validate_guess() {
        use deck::{Card, Suit, Value};
        use red_or_black::messages::CardColour;

        let game = RedOrBlack::new(vec!["mick".to_string()]);
        assert_eq!(
            game.validate_guess(
                &CardColour::Red,
                Card {
                    value: Value::Ace,
                    suit: Suit::Heart,
                }
            ),
            true
        );

        assert_eq!(
            game.validate_guess(
                &CardColour::Black,
                Card {
                    value: Value::Ace,
                    suit: Suit::Diamond,
                }
            ),
            false
        );

        assert_eq!(
            game.validate_guess(
                &CardColour::Red,
                Card {
                    value: Value::Ace,
                    suit: Suit::Spade,
                }
            ),
            false
        );

        assert_eq!(
            game.validate_guess(
                &CardColour::Black,
                Card {
                    value: Value::Ace,
                    suit: Suit::Club,
                }
            ),
            true
        );
    }
}
