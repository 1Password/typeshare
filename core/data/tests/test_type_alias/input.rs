#[typeshare]
pub struct Bar(String);

#[typeshare]
pub struct Foo {
    bar: Bar,
}
