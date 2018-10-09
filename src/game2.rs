use deck::{Card, Deck, Suit};
use messages::CardColour;


#[derive(Clone,Serialize)]
pub struct CardHistory {
    size: u16,
    // Maybe should be linked list
    history: Vec<Option<Card>>,
}

impl CardHistory {
    pub fn new(size: u16) -> Self {
        CardHistory {
            size,
            history: Vec::new(),
        }
    }

    pub fn push(&mut self, card: Card) -> &Vec<Option<Card>> {
        self.history.push(Some(card));
        if self.history.len() >= self.size as usize{
            self.history.remove(0);
        }
        &self.history
    }
}

pub struct RedOrBlack {
    usernames: Vec<String>,
    index: usize,
    penalty: u16,
    deck: Deck,
    card_history: CardHistory,
}

impl RedOrBlack {
    pub fn new(usernames: Vec<String>) -> Self {
        RedOrBlack {
            usernames,
            index: 0,
            penalty: 5,
            deck: Deck::new_shuffled(),
            card_history: CardHistory::new(3),
        }
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

        if self.usernames.len() == 0 {
            // Reset penalty
            self.penalty = 5;
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
            self.deck = Deck::new_shuffled();
            self.deck.pop().unwrap()
        }
    }

    pub fn validate_guess(&self, guess: &CardColour, card: Card) -> bool {
        // let card = self.draw_card();
        guess == &CardColour::Black && (card.suit == Suit::Spade || card.suit == Suit::Club)
            || guess == &CardColour::Red && (card.suit == Suit::Heart || card.suit == Suit::Diamond)
    }

    // validate guess, and change players turn
    pub fn play_turn(&mut self, guess: &CardColour) -> (bool, u16, Option<&String>, Card) {
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
        let player = self.next_player();
        (correct, penalty, player, card)
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    mod penalty {
        use super::*;
        use messages::CardColour;

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
        use messages::CardColour;

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

    #[test]
    fn validate_guess() {
        use deck::{Card, Suit, Value};
        use messages::CardColour;

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
