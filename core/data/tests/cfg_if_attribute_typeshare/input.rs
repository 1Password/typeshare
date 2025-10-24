/// Example of a type that is conditionally typeshared
/// based on a feature "typeshare-support". This does not
/// conditionally typeshare but allows a conditionally
/// typeshared type to generate typeshare types when behind
/// a `cfg_attr` condition.
#[cfg_attr(feature = "typeshare-support", typeshare)]
pub struct TestStruct1 {
    field: String,
}

#[cfg_attr(feature = "typeshare-support", typeshare(transparent))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
#[repr(transparent)]
pub struct Bytes(Vec<u8>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "typeshare-support",
    typeshare(
        swift = "Equatable, Hashable",
        swiftGenericConstraints = "R: Equatable & Hashable"
    )
)]
pub struct TestStruct2<R> {
    field_1: String,
    field_2: R,
}

#[cfg_attr(feature = "typeshare-support", typeshare(redacted))]
pub struct TestStruct3 {
    field_1: String,
}
