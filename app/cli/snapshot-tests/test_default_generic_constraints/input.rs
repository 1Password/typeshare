#[typeshare]
struct GenericType<K, V> {
    key: K,
    value: V
}

#[typeshare]
#[serde(tag = "type", content = "content")]
enum GenericEnum<K, V> {
    Variant {
        key: K,
        value: V
    }
}