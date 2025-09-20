use crate::{
    error::{GenerationError, ParseError},
    parser::ParsedData,
    rust_types::{
        Id, RustConst, RustEnum, RustEnumVariant, RustItem, RustStruct, RustType, RustTypeAlias,
        RustTypeFormatError, SpecialRustType,
    },
    topsort::topsort,
    visitors::ImportedType,
};
use itertools::Itertools;
use log::{debug, warn};
use proc_macro2::Ident;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
    fs,
    io::Write,
    path::Path,
    str::FromStr,
};

mod go;
mod kotlin;
mod python;
mod scala;
mod swift;
mod typescript;

pub use go::Go;
pub use kotlin::Kotlin;
pub use python::Python;
pub use scala::Scala;
pub use swift::GenericConstraints;
pub use swift::Swift;
pub use typescript::TypeScript;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
/// A crate name.
pub struct CrateName(String);

impl Display for CrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CrateName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// When using single file output we put all types into a single virtual name space.
pub const SINGLE_FILE_CRATE_NAME: CrateName = CrateName(String::new());

impl CrateName {
    /// View this crate name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Extract the crate name from a give path.
    pub fn find_crate_name(path: &Path) -> Option<Self> {
        let file_name_to_crate_name = |file_name: &str| file_name.replace('-', "_");

        debug!("Attempting to find crate name for path: {:?}", path);

        // First, find the git repository root to limit our search scope
        let git_root = Self::find_git_root(path);
        if let Some(ref root) = git_root {
            debug!("Found git repository root at: {:?}", root);
        } else {
            debug!("No git repository root found");
        }

        // First try to find crate name from path structure within git boundary
        let from_path =
            Self::find_crate_name_from_path_within_git(path, git_root.as_deref()).map(|name| {
                debug!("Found crate name from path structure: {}", name);
                CrateName::from(file_name_to_crate_name(&name).as_str())
            });

        if let Some(crate_name) = from_path {
            return Some(crate_name);
        }

        debug!("No src directory found in path, trying Cargo.toml fallback");
        // If that fails, try to read from Cargo.toml within git boundary
        let from_cargo =
            Self::find_crate_name_from_cargo_toml_within_git(path, git_root.as_deref());
        if let Some(ref crate_name) = from_cargo {
            debug!("Found crate name from Cargo.toml: {}", crate_name.as_str());
        } else {
            debug!("No crate name found from Cargo.toml");
        }
        from_cargo
    }

    /// Find the git repository root by walking up directories looking for .git
    fn find_git_root(path: &Path) -> Option<std::path::PathBuf> {
        let mut current_path = if path.is_file() {
            path.parent()?.to_path_buf()
        } else {
            path.to_path_buf()
        };

        loop {
            let git_dir = current_path.join(".git");
            if git_dir.exists() {
                debug!("Found .git directory at: {:?}", current_path);
                return Some(current_path);
            }

            // Move up one directory
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                break;
            }
        }

        None
    }

    /// Find crate name from path structure, but only within git repository boundaries
    fn find_crate_name_from_path_within_git(
        path: &Path,
        git_root: Option<&Path>,
    ) -> Option<String> {
        // Collect path components, but only those within the git repository
        let path_to_search = if let Some(git_root) = git_root {
            // Strip the git root from the path for searching
            path.strip_prefix(git_root).unwrap_or(path)
        } else {
            path
        };

        debug!(
            "Searching for 'src' directory in path relative to git root: {:?}",
            path_to_search
        );

        let path_components: Vec<_> = path_to_search.iter().collect();

        // Look for "src" in the path components (now within git boundary)
        for (i, component) in path_components.iter().enumerate().rev() {
            if component == &"src" && i > 0 {
                // Found "src", get the directory name before it
                if let Some(parent_name) = path_components[i - 1].to_str() {
                    debug!(
                        "Found 'src' at position {}, parent directory: {}",
                        i, parent_name
                    );
                    return Some(parent_name.to_string());
                }
            }
        }

        debug!("No 'src' directory found in git-bounded path");
        None
    }

    /// Find crate name from Cargo.toml, but only within git repository boundaries
    fn find_crate_name_from_cargo_toml_within_git(
        path: &Path,
        git_root: Option<&Path>,
    ) -> Option<Self> {
        let mut current_path = if path.is_file() {
            path.parent()?.to_path_buf()
        } else {
            path.to_path_buf()
        };

        // Walk up the directory tree looking for Cargo.toml, but stop at git root
        loop {
            let cargo_toml = current_path.join("Cargo.toml");
            debug!("Checking for Cargo.toml at: {:?}", cargo_toml);

            if cargo_toml.exists() {
                debug!("Found Cargo.toml at: {:?}", cargo_toml);
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    if let Some(crate_name) = Self::extract_name_from_cargo_toml(&content) {
                        debug!(
                            "Successfully extracted crate name from Cargo.toml: {}",
                            crate_name.as_str()
                        );
                        return Some(crate_name);
                    } else {
                        debug!("Failed to extract crate name from Cargo.toml content");
                    }
                } else {
                    debug!("Failed to read Cargo.toml file");
                }
            }

            // Move up one directory, but stop at git root
            if let Some(parent) = current_path.parent() {
                // If we have a git root, don't go past it
                if let Some(git_root) = git_root {
                    if parent < git_root {
                        debug!("Reached git repository boundary, stopping Cargo.toml search");
                        break;
                    }
                }
                current_path = parent.to_path_buf();
            } else {
                debug!("Reached filesystem root, stopping search");
                break;
            }
        }

        None
    }

    /// Extract the name field from a Cargo.toml content string using proper TOML parsing.
    fn extract_name_from_cargo_toml(content: &str) -> Option<Self> {
        use std::collections::HashMap;

        debug!("Parsing Cargo.toml content for package name");

        match toml::from_str::<HashMap<String, toml::Value>>(content) {
            Ok(parsed) => {
                if let Some(package) = parsed.get("package") {
                    if let Some(name_value) = package.get("name") {
                        if let Some(name) = name_value.as_str() {
                            let file_name_to_crate_name =
                                |file_name: &str| file_name.replace('-', "_");
                            debug!("Found package name in TOML: {}", name);
                            return Some(CrateName::from(file_name_to_crate_name(name).as_str()));
                        }
                    } else {
                        debug!("No 'name' field found in [package] section");
                    }
                } else {
                    debug!("No [package] section found in TOML");
                }
            }
            Err(e) => {
                debug!("Failed to parse TOML content: {}", e);
            }
        }

        None
    }
}

impl From<&str> for CrateName {
    fn from(value: &str) -> Self {
        CrateName(value.to_string())
    }
}

/// A type name.
pub type TypeName = String;

/// Mapping of crate names to typeshare type names.
pub type CrateTypes = HashMap<CrateName, HashSet<TypeName>>;

/// A sorted crate name ref.
pub type SortedCrateNames<'a> = &'a CrateName;
/// A sorted type name ref.
pub type SortedTypeNames<'a> = BTreeSet<&'a str>;

/// Reference types by crate that are scoped for a given output module.
pub type ScopedCrateTypes<'a> = BTreeMap<SortedCrateNames<'a>, SortedTypeNames<'a>>;

/// All supported programming languages.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    Go,
    Kotlin,
    Scala,
    Swift,
    TypeScript,
    Python,
}

impl SupportedLanguage {
    /// Returns an iterator over all supported language variants.
    pub fn all_languages() -> impl Iterator<Item = Self> {
        use SupportedLanguage::*;
        [Go, Kotlin, Scala, Swift, TypeScript, Python].into_iter()
    }

    /// Get the file name extension for the supported language.
    pub fn language_extension(&self) -> &'static str {
        match self {
            SupportedLanguage::Go => "go",
            SupportedLanguage::Kotlin => "kt",
            SupportedLanguage::Scala => "scala",
            SupportedLanguage::Swift => "swift",
            SupportedLanguage::TypeScript => "ts",
            SupportedLanguage::Python => "py",
        }
    }
}

impl FromStr for SupportedLanguage {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "go" => Ok(Self::Go),
            "kotlin" => Ok(Self::Kotlin),
            "scala" => Ok(Self::Scala),
            "swift" => Ok(Self::Swift),
            "typescript" => Ok(Self::TypeScript),
            "python" => Ok(Self::Python),
            _ => Err(ParseError::UnsupportedLanguage(s.into())),
        }
    }
}

impl TryFrom<&Ident> for SupportedLanguage {
    type Error = ParseError;

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Self::from_str(ident.to_string().as_str())
    }
}

/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language {
    /// Given `data`, generate type-code for this language and write it out to `writable`.
    /// Returns whether or not writing was successful.
    fn generate_types(
        &mut self,
        writable: &mut dyn Write,
        all_types: &CrateTypes,
        data: ParsedData,
    ) -> std::io::Result<()> {
        self.begin_file(writable, &data)?;

        if data.multi_file {
            self.write_imports(writable, used_imports(&data, all_types))?;
        }

        let ParsedData {
            structs,
            enums,
            aliases,
            consts,
            ..
        } = data;

        let mut items = Vec::from_iter(
            aliases
                .into_iter()
                .map(RustItem::Alias)
                .chain(structs.into_iter().map(RustItem::Struct))
                .chain(enums.into_iter().map(RustItem::Enum))
                .chain(consts.into_iter().map(RustItem::Const)),
        );

        topsort(&mut items);

        for thing in &items {
            match thing {
                RustItem::Enum(e) => self.write_enum(writable, e)?,
                RustItem::Struct(s) => self.write_struct(writable, s)?,
                RustItem::Alias(a) => self.write_type_alias(writable, a)?,
                RustItem::Const(c) => self.write_const(writable, c)?,
            }
        }

        self.end_file(writable)
    }

    /// Get the type mapping for this language `(Rust type name -> lang type name)`
    fn type_map(&mut self) -> &HashMap<String, String>;

    /// Convert a Rust type into a type from this language.
    fn format_type(
        &mut self,
        ty: &RustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        match ty {
            RustType::Simple { id } => self.format_simple_type(id, generic_types),
            RustType::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_types)
            }
            RustType::Special(special) => self.format_special_type(special, generic_types),
        }
    }

    // We need to pass in an &String for type mapping
    /// Format a simple type with no generic parameters.
    /// Note that we still need to take a list of generic types in case the implementors
    /// need to differentiate between a user-defined type and a generic type (for example: Swift)
    #[allow(clippy::ptr_arg)]
    fn format_simple_type(
        &mut self,
        base: &String,
        _generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(if let Some(mapped) = self.type_map().get(base) {
            mapped.into()
        } else {
            base.into()
        })
    }

    // We need to pass in an &String for type mapping
    /// Format a generic type that takes in generic arguments, which
    /// may be recursive.
    #[allow(clippy::ptr_arg)]
    fn format_generic_type(
        &mut self,
        base: &String,
        parameters: &[RustType],
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_map().get(base) {
            Ok(mapped.into())
        } else {
            let parameters: Result<Vec<String>, RustTypeFormatError> = parameters
                .iter()
                .map(|p| self.format_type(p, generic_types))
                .collect();
            let parameters = parameters?;
            Ok(format!(
                "{}{}",
                self.format_simple_type(base, generic_types)?,
                if !parameters.is_empty() {
                    self.format_generic_parameters(parameters)
                } else {
                    Default::default()
                }
            ))
        }
    }

    /// Format generic parameters into A<T,R> which is common for many supported languages.
    /// Reimplement if other notations is used.
    fn format_generic_parameters(&mut self, parameters: Vec<String>) -> String {
        format!("<{}>", parameters.into_iter().join(", "))
    }

    /// Format a base type that is classified as a SpecialRustType.
    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError>;

    /// Implementors can use this function to write a header for typeshared code
    fn begin_file(&mut self, _w: &mut dyn Write, _parsed_data: &ParsedData) -> std::io::Result<()> {
        Ok(())
    }

    /// For generating import statements.
    fn write_imports(
        &mut self,
        _writer: &mut dyn Write,
        _imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()>;

    /// Implementors can use this function to write a footer for typeshared code
    fn end_file(&mut self, _w: &mut dyn Write) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a type alias by converting it.
    /// Example of a type alias:
    /// ```
    /// type MyTypeAlias = String;
    /// ```
    fn write_type_alias(&mut self, _w: &mut dyn Write, _t: &RustTypeAlias) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a constant variable.
    /// Example of a constant variable:
    /// ```
    /// const ANSWER_TO_EVERYTHING: u32 = 42;
    /// ```
    fn write_const(&mut self, _w: &mut dyn Write, _c: &RustConst) -> std::io::Result<()> {
        Ok(())
    }

    /// Write a struct by converting it
    /// Example of a struct:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// struct Foo {
    ///     bar: String
    /// }
    /// ```
    fn write_struct(&mut self, _w: &mut dyn Write, _rs: &RustStruct) -> std::io::Result<()> {
        Ok(())
    }

    /// Write an enum by converting it.
    /// Example of an enum:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// #[serde(tag = "type", content = "content")]
    /// enum Foo {
    ///     Fizz,
    ///     Buzz { yep_this_works: bool }
    /// }
    /// ```
    fn write_enum(&mut self, _w: &mut dyn Write, _e: &RustEnum) -> std::io::Result<()> {
        Ok(())
    }

    /// Write out named types to represent anonymous struct enum variants.
    ///
    /// Take the following enum as an example:
    ///
    /// ```
    /// enum AlgebraicEnum {
    ///     AnonymousStruct {
    ///         field: String,
    ///         another_field: bool,
    ///     },
    /// }
    /// ```
    ///
    /// This function will write out:
    ///
    /// ```rust
    /// /// Generated type representing the anonymous struct variant `<make_struct_name>` of the `AlgebraicEnum` rust enum
    /// /* the struct definition for whatever language */
    /// ```
    ///
    /// It does this by calling `write_struct` on the given `language_impl`. The
    /// name of the struct is controlled by the `make_struct_name` closure; you're
    /// given the variant name and asked to return whatever struct name works best
    /// for your language.
    fn write_types_for_anonymous_structs(
        &mut self,
        w: &mut dyn Write,
        e: &RustEnum,
        make_struct_name: &dyn Fn(&str) -> String,
    ) -> std::io::Result<()> {
        for (fields, shared) in e.shared().variants.iter().filter_map(|v| match v {
            RustEnumVariant::AnonymousStruct { fields, shared } => Some((fields, shared)),
            _ => None,
        }) {
            let struct_name = make_struct_name(&shared.id.original);

            // Builds the list of generic types (e.g [T, U, V]), by digging
            // through the fields recursively and comparing against the
            // enclosing enum's list of generic parameters.
            let generic_types = fields
                .iter()
                .flat_map(|field| {
                    e.shared()
                        .generic_types
                        .iter()
                        .filter(|g| field.ty.contains_type(g))
                })
                .unique()
                .cloned()
                .collect();

            self.write_struct(
                w,
                &RustStruct {
                    id: Id {
                        original: struct_name.clone(),
                        renamed: struct_name.clone(),
                        serde_rename: false
                    },
                    fields: fields.clone(),
                    generic_types,
                    comments: vec![format!(
                        "Generated type representing the anonymous struct variant `{}` of the `{}` Rust enum",
                        &shared.id.original,
                        &e.shared().id.original,
                    )],
                    decorators: e.shared().decorators.clone(),
                    is_redacted: e.shared().is_redacted,
                },
            )?;
        }

        Ok(())
    }

    /// Types that are remapped will be excluded from import references.
    fn ignored_reference_types(&self) -> Vec<&str> {
        Vec::new()
    }

    /// Any other final steps after modules have been generated. For example creating a new
    /// module with special types.
    fn post_generation(&self, _output_folder: &str) -> Result<(), GenerationError> {
        Ok(())
    }
}

/// Lookup any refeferences to other typeshared types in order to build
/// a list of imports for the generated module.
fn used_imports<'a, 'b: 'a>(
    data: &'b ParsedData,
    all_types: &'a CrateTypes,
) -> ScopedCrateTypes<'a> {
    let mut used_imports = BTreeMap::new();

    // If we have reference that is a re-export we can attempt to find it with the
    // following heuristic.
    let fallback = |referenced_import: &'a ImportedType, used: &mut ScopedCrateTypes<'a>| {
        // Find the first type that does not belong to the current crate.
        if let Some((crate_name, ty)) = all_types
            .iter()
            .flat_map(|(k, v)| {
                v.iter()
                    .find(|&t| t == &referenced_import.type_name && k != &data.crate_name)
                    .map(|t| (k, t))
            })
            .next()
        {
            warn!("Warning: Using {crate_name} as module for {ty} which is not in referenced crate {}", referenced_import.base_crate);
            used.entry(crate_name)
                .and_modify(|v| {
                    v.insert(ty.as_str());
                })
                .or_insert(BTreeSet::from([ty.as_str()]));
        } else {
            // println!("Could not lookup reference {referenced_import:?}");
        }
    };

    for referenced_import in data
        .import_types
        .iter()
        // Skip over imports that reference the current crate. They
        // are all collapsed into one module per crate.
        .filter(|imp| imp.base_crate != data.crate_name)
    {
        // Look up the types for the referenced imported crate.
        if let Some(type_names) = all_types.get(&referenced_import.base_crate) {
            if referenced_import.type_name == "*" {
                // We can have "*" wildcard here. We need to add all.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .and_modify(|names: &mut BTreeSet<&str>| {
                        names.extend(type_names.iter().map(|s| s.as_str()))
                    });
            } else if let Some(ty_name) = type_names.get(&referenced_import.type_name) {
                // Add referenced import for each matching type.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .and_modify(|v| {
                        v.insert(ty_name.as_str());
                    })
                    .or_insert(BTreeSet::from([ty_name.as_str()]));
            } else {
                fallback(referenced_import, &mut used_imports);
            }
        } else {
            // We might have a re-export from another crate.
            fallback(referenced_import, &mut used_imports);
        }
    }
    used_imports
}

#[cfg(test)]
mod test {
    use crate::language::CrateName;
    use std::path::Path;

    #[test]
    fn test_crate_name() {
        let path = Path::new("/some/path/to/projects/core/foundation/op-proxy/src/android.rs");
        assert_eq!(Some("op_proxy".into()), CrateName::find_crate_name(path));
    }
}
