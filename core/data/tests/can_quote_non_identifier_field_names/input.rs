#[typeshare]
pub struct LinkedData {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "schema.org")]
    pub schema_org: String,
    #[serde(rename = "2fa")]
    pub two_factor: bool,
    #[serde(rename = "$ref")]
    pub reference: String,
}
