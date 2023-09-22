use typeshare_typescript::{execute_inner, Commands};

fn main() {
    let command = Commands::parse_and_handle_extras();
    execute_inner(command);
}
