use bevy::prelude::*;

#[derive(Reflect, Clone, Copy, Debug)]
pub enum CardSuit {
    Hearts,
    Spades,
    Diamonds,
    Clubs,
}
#[derive(Reflect, Clone, Copy, Debug)]
pub enum CardColor {
    Red,
    Black,
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
pub struct Card {
    pub index: usize,
    pub number: usize,
    pub suit: CardSuit,
    pub color: CardColor,
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
pub struct CardSlot;

#[derive(Component)]
pub struct CardFront;
#[derive(Component)]
pub struct CardBack;

#[derive(Resource, Reflect, Default)]
pub struct Cards {
    pub cards: Vec<Card>,
}

#[derive(Resource)]
pub struct CurrentCard(pub usize);

pub const CARD_SIZE: Vec2 = Vec2::new(53.0, 70.0);

