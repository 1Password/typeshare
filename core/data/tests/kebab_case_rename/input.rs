/// This is a comment.
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Things {
    pub bla: String,
    #[serde(rename = "label")]
    pub some_label: Option<String>,
    #[serde(rename = "label-left")]
    pub label_left: Option<String>,
}
