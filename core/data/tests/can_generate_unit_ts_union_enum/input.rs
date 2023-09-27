#[typeshare(ts_union)]
pub enum UnitEnumMultiple {
    VariantA,
    VariantB,
    VariantC,
}

#[typeshare(ts_union)]
pub enum UnitEnumOne {
    VariantA,
}

#[typeshare(ts_union)]
pub enum UnitEnumSkip {
    #[typeshare(skip)]
    VariantA,
    VariantB,
}
