#[typeshare]
pub struct MyStruct {
    a: i32,
    #[serde(skip)]
    b: i32,
    c: i32,
    #[typeshare(skip)]
    d: i32,
}
