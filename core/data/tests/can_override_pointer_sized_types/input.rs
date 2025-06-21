#[typeshare]
struct PointerSizedType {
    #[typeshare(kotlin(type = "ULong"))]
    #[typeshare(scala(type = "ULong"))]
    #[typeshare(swift(type = "UInt64"))]
    #[typeshare(typescript(type = "number"))]
    #[typeshare(go(type = "uint64"))]
    unsigned: usize,
    #[typeshare(kotlin(type = "Long"))]
    #[typeshare(scala(type = "Long"))]
    #[typeshare(swift(type = "Int64"))]
    #[typeshare(typescript(type = "number"))]
    #[typeshare(go(type = "int64"))]
    signed: isize,
}
