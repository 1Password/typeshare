#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams {
    PittsburghPenguins,
    Lies(String),
}
#[typeshare(swift = "Equatable")]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams1 {
    PittsburghPenguins,
    Lies(String),
}

#[typeshare(swift = "Equatable, Codable, Comparable, Hashable")]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams2 {
    PittsburghPenguins,
    Lies(String),
}

#[typeshare(kotlin = "idk")]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams3 {
    PittsburghPenguins,
    Lies(String),
}

#[typeshare(swift = "Equatable", swift = "Hashable")]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams4 {
    PittsburghPenguins,
    Lies(String),
}
