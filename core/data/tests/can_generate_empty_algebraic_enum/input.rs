#[typeshare]
pub struct AddressDetails {}

#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum Address {
    FixedAddress(AddressDetails),
    NoFixedAddress,
}
