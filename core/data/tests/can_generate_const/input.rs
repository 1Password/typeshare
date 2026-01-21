#[typeshare]
pub const MY_INT_VAR: u32 = 12;

// String literal-related consts below:

#[typeshare]
pub const EMPTY: &'static str = "";

#[typeshare]
pub const SIMPLE_ASCII: &'static str = "Hello, world!";

#[typeshare]
pub const MULTILINE: &'static str = "Line1
Line2
Line3";

#[typeshare]
pub const ESCAPED_CHARACTERS: &'static str = "First\\line.\nSecond \"quoted\" line.\tEnd.";

#[typeshare]
pub const UNICODE: &'static str = "Emoji: 😄, Accented: café, Chinese: 世界";

#[typeshare]
pub const RAW_STRING: &'static str = r#"Raw \n, "quotes" are okay, and single \ is fine too"#;

#[typeshare]
pub const CONTAINS_BACKTICK: &'static str = "Backtick: ` inside";

#[typeshare]
pub const CONTAINS_DOLLAR_CURLY: &'static str = "${not_interpolation}";

#[typeshare]
pub const ENDS_WITH_ODD_BACKSLASH: &'static str = r"Odd number of backslashes: \\\";

#[typeshare]
pub const NULL_BYTE: &'static str = "Null:\0End";

#[typeshare]
pub const COMBINING: &'static str = "e\u{301} vs é"; // normalization check
