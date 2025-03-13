/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub enum BoxyColors {
    Red,
    Blue,
    Green(Box<String>),
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct ArcyColors {
    pub red: Weak<u8>,
    pub blue: ArcWeak<String>,
    pub green: Arc<Vec<String>>,
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct MutexyColors {
    pub blue: Mutex<Vec<String>>,
    pub green: Mutex<String>,
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct RcyColors {
    pub red: RcWeak<String>,
    pub blue: Rc<Vec<String>>,
    pub green: Rc<String>,
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct CellyColors {
    pub red: Cell<String>,
    pub blue: RefCell<Vec<String>>,
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct LockyColors {
    pub red: RwLock<String>,
}

/// This is a comment.
#[typeshare]
#[serde(tag = "type", content = "content")]
pub struct CowyColors<'a> {
    pub lifetime: Cow<'a, str>,
}
