mod a {
    #[typeshare]
    pub struct A {
        field: u32
    }
    mod b {
        mod c {
            #[typeshare]
            pub struct ABC {
                field: u32
            }
        }
        #[typeshare]
        pub struct AB {
            field: u32
        }
    }
}

#[typeshare]
pub struct OutsideOfModules {
    field: u32
}