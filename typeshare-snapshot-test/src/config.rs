use std::{fs, io, path::Path, str};

use anyhow::Context;
use toml::{Table, Value, map::Entry};

/// Recursively merge a pair of tables. For each key in `specialized`; it is
/// inserted into the table; if both sides are a table, they are merged;
/// if both sides are an array, the specialized array is appended to the base
/// one.
pub fn merge_configs(base: &mut Table, specialized: Table) {
    for (key, value) in specialized {
        match base.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
            Entry::Occupied(entry) => merge_values(entry.into_mut(), value),
        }
    }
}

/// Recursively merge a pair of toml Values. If both values are tables, they
/// are merged recursively; if both values are arrays, the specialized array
/// is appended to the base array. In all other cases, the specialized value
/// replaces the base value.
pub fn merge_values(base: &mut Value, specialized: Value) {
    match (base, specialized) {
        (Value::Table(base), Value::Table(specialized)) => merge_configs(base, specialized),
        (Value::Array(base), Value::Array(specialized)) => base.extend(specialized),
        (base, primitive) => *base = primitive,
    }
}

/// Read a toml file from the given path, returning an empty table if there
/// is no file at that path.
pub fn read_toml(path: &Path) -> anyhow::Result<Table> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Table::new()),
        err @ Err(_) => err.context("i/o error")?,
    };

    let content = str::from_utf8(&content).context("file wasn't valid UTF-8")?;
    toml::from_str(content).context("failed to parse file as toml")
}
