use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Card {
    value: Value,
    suit: Suit,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum Value {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum Suit {
    Spade,
    Club,
    Heart,
    Diamond,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        use self::Suit::*;
        use self::Value::*;

        let mut cards = Vec::new();

        for suit in &[Spade, Club, Diamond, Heart] {
            for value in &[
                Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King,
            ] {
                cards.push(Card {
                    value: *value,
                    suit: *suit,
                });
            }
        }
        Deck { cards }
    }

    pub fn new_shuffled() -> Self {
        let mut deck = Self::new();
        thread_rng().shuffle(deck.cards.as_mut_slice());
        deck
    }

    pub fn shuffle(&mut self) {
        thread_rng().shuffle(self.cards.as_mut_slice());
    }

    pub fn peek(&self) -> Option<&Card> {
        self.cards.get(0)
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}
