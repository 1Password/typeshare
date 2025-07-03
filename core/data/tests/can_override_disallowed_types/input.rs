#[typeshare]
struct DisallowedType {
    #[typeshare(typescript(type = "bigint"))]
    disallowed_type: u64,
    #[typeshare(typescript(type = "number"))]
    another_disallowed_type: i64,
    #[typeshare(typescript(type = "string"))]
    #[serde(with = "my_string_serde_impl")]
    disallowed_type_serde_with: u64,
}
