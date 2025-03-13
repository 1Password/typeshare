#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct E {
    depends_on: D,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct D {
    depends_on: C,
    also_depends_on: Option<E>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct C {
    depends_on: B
}

#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct B {
    depends_on: A,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct A {
    field: u32
}

