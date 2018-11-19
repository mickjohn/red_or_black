use std::collections::VecDeque;
use deck::Card;
use super::messages::CardColour;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HistoryItem {
    pub username: String,
    pub guess: CardColour,
    pub outcome: bool,
    pub card: Card,
    pub penalty: u16,
    pub turn_number: u16,
}

pub struct GameHistory {
    size: u16,
    history: Vec<HistoryItem>,
}

impl GameHistory {
    pub fn new(size: u16) -> Self {
        GameHistory {
            size,
            history: Vec::new(),
        }
    }

    pub fn push(&mut self, item: HistoryItem) -> &Vec<HistoryItem> {
        self.history.push(item);
        if self.history.len() > self.size as usize {
            self.history.remove(0);
        }
        &self.history
    }

    pub fn get_history(&self) -> &Vec<HistoryItem> {
        &self.history
    }
}

#[derive(Clone, Serialize)]
pub struct CardHistory {
    size: u16,
    history: VecDeque<Option<Card>>,
}

impl CardHistory {
    pub fn new(size: u16) -> Self {
        let mut vdq = VecDeque::with_capacity(size as usize);
        for _ in 0..size {
            vdq.push_front(None);
        }

        CardHistory { size, history: vdq }
    }

    pub fn push(&mut self, card: Card) -> &VecDeque<Option<Card>> {
        self.history.push_front(Some(card));
        if self.history.len() >= self.size as usize {
            self.history.pop_back();
        }
        &self.history
    }

    pub fn get_history(&self) -> &VecDeque<Option<Card>> {
        &self.history
    }
}

#[cfg(test)]
mod game_history {
    use super::*;
    use deck::*;

    #[test]
    fn can_push_onto_history() {
        let mut game_history = GameHistory::new(3);
        assert!(game_history.get_history().is_empty());
        let item = HistoryItem {
            username: "Jimmy".to_string(),
            guess: CardColour::Red,
            outcome: true,
            card: Card {
                value: Value::Ace,
                suit: Suit::Club,
            },
            penalty: 5,
            turn_number: 1,
        };
        game_history.push(item);
        assert_eq!(game_history.get_history().len(), 1);
    }

    #[test]
    fn history_is_truncated() {
        let mut game_history = GameHistory::new(3);
        let item = HistoryItem {
            username: "Jimmy".to_string(),
            guess: CardColour::Red,
            outcome: true,
            card: Card {
                value: Value::Ace,
                suit: Suit::Club,
            },
            penalty: 5,
            turn_number: 1,
        };
        game_history.push(item.clone());
        game_history.push(item.clone());
        game_history.push(item.clone());
        game_history.push(item);
        assert_eq!(game_history.get_history().len(), 3);
    }

    #[test]
    fn old_items_are_truncated_first() {
        let mut game_history = GameHistory::new(3);
        let old_item = HistoryItem {
            username: "Jimmy".to_string(),
            guess: CardColour::Red,
            outcome: true,
            card: Card {
                value: Value::Ace,
                suit: Suit::Club,
            },
            penalty: 5,
            turn_number: 1,
        };

        let new_item = HistoryItem {
            username: "Jimmy newtron".to_string(),
            guess: CardColour::Red,
            outcome: false,
            card: Card {
                value: Value::Ace,
                suit: Suit::Club,
            },
            penalty: 5,
            turn_number: 1,
        };
        game_history.push(old_item.clone());
        game_history.push(new_item.clone());
        game_history.push(new_item.clone());
        assert_eq!(game_history.get_history()[0], old_item);
        game_history.push(new_item.clone());
        assert_eq!(game_history.get_history()[0], new_item);
    }
}

mod card_history {
    use super::CardHistory;
    use deck::Deck;

    #[test]
    fn can_push_onto_history() {
        let mut history = CardHistory::new(3);
        assert_eq!(history.get_history().len(), 3);
        let mut deck = Deck::new();
        let card = deck.pop().unwrap();
        history.push(card.clone());
        // Len is fixed size, should still be same
        assert_eq!(history.get_history().len(), 3);
        assert_eq!(history.get_history()[0], Some(card));
        assert_eq!(history.get_history()[1], None);
        assert_eq!(history.get_history()[2], None);
    }

    #[test]
    fn history_is_truncated() {
        let mut history = CardHistory::new(3);
        let mut deck = Deck::new();
        let card1 = deck.pop().unwrap();
        let card2 = deck.pop().unwrap();
        let card3 = deck.pop().unwrap();
        let card4 = deck.pop().unwrap();
        history.push(card1.clone());
        history.push(card2.clone());
        history.push(card3.clone());
        assert_eq!(history.get_history()[0], Some(card3));
        assert_eq!(history.get_history()[1], Some(card2));
        assert_eq!(history.get_history()[2], Some(card1));
        history.push(card4.clone());
        assert_eq!(history.get_history()[0], Some(card4));
        assert_eq!(history.get_history()[1], Some(card3));
        assert_eq!(history.get_history()[2], Some(card2));
    }
}

