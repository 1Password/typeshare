#[typeshare]
#[serde(transparent)]
pub struct Players {
    players: Vec<Player>,
}

#[typeshare]
pub struct Player {
    name: String,
}

#[typeshare]
#[serde(transparent)]
pub struct Pool {
    cards: Cards,
    #[serde(skip)]
    queue: Vec<Card>,
}

#[typeshare]
pub struct Cards(Vec<Card>);

#[typeshare]
pub struct Card {
    name: String,
}