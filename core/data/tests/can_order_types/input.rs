#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum SomeEnum {
    Unit,
    Alg(SomeStruct),
    Anon {the_field: SomeStruct},
}

#[typeshare]
type SomeTypeAlias = SomeStruct<String>;

#[typeshare]
pub struct SomeStruct<T> {
    some_field: SomeEnum,
    some_special_field: Vec<Option<HashMap<T, SomeEnum>>>,
}
