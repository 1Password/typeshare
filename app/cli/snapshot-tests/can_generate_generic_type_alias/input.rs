#[typeshare]
type GenericTypeAlias<T> = Vec<T>;

#[typeshare]
type NonGenericAlias = GenericTypeAlias<Option<String>>;
