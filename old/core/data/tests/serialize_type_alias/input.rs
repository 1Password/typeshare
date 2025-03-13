#[typeshare]
type AlsoString = String;

#[typeshare(serialized_as = "String")]
struct Uuid(String);

#[typeshare]
/// Unique identifier for an Account
type AccountUuid = Uuid;

#[typeshare(serialized_as = "String")]
type ItemUuid = Uuid;
