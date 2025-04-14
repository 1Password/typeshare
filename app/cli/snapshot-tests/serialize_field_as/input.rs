#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Context {
    pub urls: Vec<EditItemContextUrl>,
    pub apps: Vec<ItemApp>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
pub struct EditItemViewModelSaveRequest {
    #[typeshare(serialized_as = "String")]
    pub(super) context: Context,

    pub values: Vec<EditItemSaveValue>,
    pub fill_action: Option<AutoFillItemActionRequest>,
}
