#[typeshare]
pub struct BestHockeyTeams {
    PittsburghPenguins: u32,
    Lies: String,
}
#[typeshare(swift = "Equatable")]
pub struct BestHockeyTeams1 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[typeshare(swift = "Equatable, Codable, Comparable, Hashable")]
pub struct BestHockeyTeams2 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[typeshare(redacted)]
pub struct BestHockeyTeams3 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[typeshare(swift = "Equatable", swift = "Hashable")]
pub struct BestHockeyTeams4 {
    PittsburghPenguins: u32,
    Lies: String,
}

#[typeshare(kotlin = "JvmInline", redacted)]
pub struct BestHockeyTeams5(String);
