#[typeshare]
pub struct catch {
    pub default: String,
    pub case: String,
}

#[typeshare]
pub enum throws {
    case,
    default,
}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum switch {
    default(catch),
}
