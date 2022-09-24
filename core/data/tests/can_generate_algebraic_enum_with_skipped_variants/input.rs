#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum SomeEnum {
    A,
    #[typeshare(skip)]
    B,
    C(i32),
    #[typeshare(skip, asdf)]
    D(u32),
    #[serde(skip)]
    E,
}
