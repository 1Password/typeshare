#[typeshare]
struct QualifiedTypes {
    unqualified: String,
    qualified: std::string::String,
    qualified_vec: Vec<std::string::String>,
    qualified_hashmap: HashMap<std::string::String, std::string::String>,
    qualified_optional: Option<std::string::String>,
    qualfied_optional_hashmap_vec: Option<HashMap<std::string::String, Vec<std::string::String>>>
}