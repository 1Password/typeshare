#[cfg(test)]
mod tests {
    use typeshare::typeshare;

    #[test]
    fn test_c_cpp_struct_generation() {
        #[typeshare]
        struct MyStruct {
            field1: i32,
            field2: String,
        }

        // Generate C code
        let c_code = typeshare::codegen::<crate::core::language::C, _>(&MyStruct::definition()).unwrap();
        println!("C code:\\n{}", c_code);

        // Generate C++ code
        let cpp_code = typeshare::codegen::<crate::core::language::Cpp, _>(&MyStruct::definition()).unwrap();
        println!("C++ code:\\n{}", cpp_code);

        assert!(c_code.contains("typedef struct MyStruct"));
        assert!(cpp_code.contains("struct MyStruct"));
    }
}