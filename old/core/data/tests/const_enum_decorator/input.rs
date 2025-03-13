#[typeshare]
pub enum BestHockeyTeams {
    PittsburghPenguins,
}

#[typeshare(swift = "Equatable")]
pub enum BestHockeyTeams1 {
    PittsburghPenguins,
}

#[typeshare(swift = "Equatable, Comparable, Hashable")]
pub enum BestHockeyTeams2 {
    PittsburghPenguins,
}

#[typeshare(kotlin = "idk")]
pub enum BestHockeyTeams3 {
    PittsburghPenguins,
}
#[typeshare(swift = "Equatable", swift = "Hashable")]
pub enum BestHockeyTeams4 {
    PittsburghPenguins,
}
