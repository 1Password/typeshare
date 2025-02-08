#[derive(Debug, Clone, Copy)]
pub enum ArgType {
    /// The argument is a switch; typically this is associated with a bool field
    Switch,

    /// The argument is takes a value, like a number, string, or path.
    Value,
}

pub trait ConfigCliArgs<'a> {
    const ARGS: &'static [(&'static str, ArgType)];

    fn enable_cli_switch(&mut self, arg: &str);
    fn apply_cli_value(&mut self, arg: &str, value: &'a str) -> anyhow::Result<()>;
}
