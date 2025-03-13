#[typeshare]
pub struct Location {}

/// This is a comment.
#[typeshare]
pub struct Person {
    /** This is another comment */
    pub name: String,
    pub age: u8,
    pub info: Option<String>,
    pub emails: Vec<String>,
    pub location: Location,
}
