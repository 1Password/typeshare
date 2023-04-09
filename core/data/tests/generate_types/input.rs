#[typeshare]
pub struct CustomType {}

#[typeshare]
pub struct Types {
    pub s: String,
    pub static_s: &'static str,
    pub int8: i8,
    pub float: f32,
    pub double: f64,
    pub array: Vec<String>,
    pub fixed_length_array: [String; 4],
    pub dictionary: HashMap<String, i32>,
    pub optional_dictionary: Option<HashMap<String, i32>>,
    pub custom_type: CustomType,
}
