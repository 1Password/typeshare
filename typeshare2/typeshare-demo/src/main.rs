use std::path::Path;

use ignore::{types::TypesBuilder, WalkBuilder};
use typeshare_engine::{parse_input, parser_inputs, write_generated, LangConfig};
use typeshare_model::prelude::FilesMode;

fn main() {
    let mut typescript = typeshare_typescript::TypeScript::default();

    let mut types = TypesBuilder::new();
    types.add("rust", "*.rs").unwrap();
    types.select("rust");

    let mut walker = WalkBuilder::new(".");
    walker.types(types.build().unwrap());

    let walker = walker.build();

    let inputs = parser_inputs(
        walker,
        &LangConfig {
            extension: ".ts",
            pascal: true,
        },
        false,
    );

    let parsed = parse_input(inputs, &[], FilesMode::Single).unwrap();
    // TODO: check errors here
    write_generated(
        typeshare_engine::OutputMode::SingleFile {
            file: Path::new("./demo.ts"),
        },
        &mut typescript,
        parsed,
    )
    .unwrap();
}
