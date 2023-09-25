use typeshare_annotation::typeshare;

#[derive(Default)]
#[typeshare]
pub struct Test {
    pub id: u32,
    pub name: String,
    #[typeshare(skip)]
    pub password: String,
}

#[typeshare]
pub enum TestEnum {
    One {
        #[typeshare(rename = "username")]
        username: String,
    },
    #[typeshare(rename = "two")]
    Two { name: String },
}
#[test]
pub fn test() {
    let _test: Test = Default::default();
}
