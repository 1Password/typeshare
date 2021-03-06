use std::io::Write;

use crate::language::{Language, Params, RustAlgebraicEnum, RustConstEnum, RustStruct};

#[derive(Default)]
pub struct Swift {}

impl Swift {
    pub fn new() -> Self {
        Swift {}
    }
}

fn swift_type(s: &str) -> &str {
    match s {
        "String" => "String",
        "i8" => "Int8",
        "i16" => "Int16",
        "i32" => "Int32",
        "i64" => "Int64",
        "u8" => "UInt8",
        "u16" => "UInt16",
        "u32" => "UInt32",
        "u64" => "UInt64",
        "isize" => "Int",
        "usize" => "UInt",
        "bool" => "Bool",
        "f32" => "Float",
        "f64" => "Double",
        _ => s,
    }
}

fn swift_lit_type(lit: &Option<syn::Lit>) -> &'static str {
    match lit {
        Some(syn::Lit::Int(_)) => "Int",
        Some(syn::Lit::Str(_)) => "String",
        Some(syn::Lit::ByteStr(_)) => "[UInt8]",
        Some(syn::Lit::Byte(_)) => "UInt8",
        Some(syn::Lit::Char(_)) => "Int8",
        Some(syn::Lit::Float(_)) => "Float",
        Some(syn::Lit::Bool(_)) => "Bool",
        Some(syn::Lit::Verbatim(_)) => " ERROR ",
        None => "String", // Should be used when we have a bare enum
    }
}

impl Language for Swift {
    fn begin_file(&mut self, w: &mut dyn Write, _params: &Params) -> std::io::Result<()> {
        writeln!(w, "/*")?;
        writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
        writeln!(w, "*/")?;

        writeln!(w)?;
        writeln!(w, "import Foundation\n")?;
        Ok(())
    }

    fn write_struct(&mut self, w: &mut dyn Write, params: &Params, rs: &RustStruct) -> std::io::Result<()> {
        write_comments(w, 0, &rs.comments)?;
        writeln!(w, "public struct {}{}: Codable {{", params.swift_prefix, rs.id.original)?;

        for f in rs.fields.iter() {
            write_comments(w, 1, &f.comments)?;
            if f.is_vec {
                writeln!(w, "\tpublic let {}: [{}]{}", f.id.renamed, swift_type(&f.ty), option_symbol(f.is_optional))?;
            } else {
                writeln!(w, "\tpublic let {}: {}{}", f.id.renamed, swift_type(&f.ty), option_symbol(f.is_optional))?;
            }
        }

        let mut init_params: Vec<String> = Vec::new();
        for f in rs.fields.iter() {
            if f.is_vec {
                init_params.push(format!("{}: [{}]{}", f.id.renamed, swift_type(&f.ty), option_symbol(f.is_optional)));
            } else {
                init_params.push(format!("{}: {}{}", f.id.renamed, swift_type(&f.ty), option_symbol(f.is_optional)));
            }
        }

        writeln!(w, "\n\tpublic init({}) {{", init_params.join(", "))?;
        for f in rs.fields.iter() {
            writeln!(w, "\t\tself.{} = {}", f.id.renamed, f.id.renamed)?;
        }
        writeln!(w, "\t}}")?;
        writeln!(w, "}}\n")?;

        write_struct_convenience_methods(w, params, rs)?;

        Ok(())
    }

    fn write_const_enum(&mut self, w: &mut dyn Write, params: &Params, e: &RustConstEnum) -> std::io::Result<()> {
        write_comments(w, 0, &e.comments)?;
        writeln!(w, "public enum {}{}: {}, Codable {{", params.swift_prefix, e.id.original, swift_lit_type(&e.ty))?;

        for c in e.cases.iter() {
            write_comments(w, 1, &c.comments)?;
            let mut printed_value = lit_value(&c.value).to_string();
            if printed_value == "" {
                printed_value = format!(r##""{}""##, &c.id.renamed);
            }

            writeln!(w, "\tcase {} = {}", c.id.renamed, &printed_value)?;
        }

        writeln!(w, "}}\n")?;
        Ok(())
    }

    fn write_algebraic_enum(&mut self, w: &mut dyn Write, params: &Params, e: &RustAlgebraicEnum) -> std::io::Result<()> {
        write_comments(w, 0, &e.comments)?;
        let enum_type_name = format!("{}{}", params.swift_prefix, e.id.original);
        writeln!(w, "public enum {}: Codable {{", enum_type_name)?;

        let mut decoding_cases: Vec<String> = Vec::new();
        let mut encoding_cases: Vec<String> = Vec::new();

        for c in e.cases.iter() {
            write_comments(w, 1, &c.comments)?;
            let case_type = if c.value.is_vec {
                format!("[{}{}]", swift_type(&c.value.ty), option_symbol(c.value.is_optional))
            } else {
                format!("{}{}", swift_type(&c.value.ty), option_symbol(c.value.is_optional))
            };

            writeln!(w, "\tcase {}({})", c.id.renamed, case_type)?;

            decoding_cases.push(format!(
                "
		if let x = try? container.decode({}.self) {{
			self = .{}(x)
			return
		}}",
                case_type, c.id.renamed,
            ));

            encoding_cases.push(format!(
                "
		case .{}(let x):
			try container.encode(x)",
                c.id.renamed,
            ));
        }

        writeln!(
            w,
            r#"
	public init(from decoder: Decoder) throws {{
		let container = try decoder.singleValueContainer(){decoding_switch}
		throw DecodingError.typeMismatch({type_name}.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for {type_name}"))
	}}

	public func encode(to encoder: Encoder) throws {{
		var container = encoder.singleValueContainer()
		switch self {{{encoding_switch}
		}}
	}}"#,
            type_name = enum_type_name,
            decoding_switch = decoding_cases.join(""),
            encoding_switch = encoding_cases.join(""),
        )?;

        writeln!(w, "}}\n")?;
        Ok(())
    }
}

fn bool_literal(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

fn option_symbol(optional: bool) -> &'static str {
    if optional {
        "?"
    } else {
        ""
    }
}

fn write_struct_convenience_methods(w: &mut dyn Write, generator_params: &Params, rs: &RustStruct) -> std::io::Result<()> {
    let data_init_params = rs
        .fields
        .iter()
        .map(|f| format!("{param}: decoded.{param}", param = f.id.renamed))
        .collect::<Vec<String>>()
        .join(", ");

    writeln!(
        w,
        "
public extension {prefix}{struct} {{
	init(data: Data) throws {{
		let decoded = try JSONDecoder().decode({prefix}{struct}.self, from: data)
		self.init({params})
	}}
}}
",
        prefix = generator_params.swift_prefix, struct = rs.id.original, params = data_init_params
    )?;

    Ok(())
}

fn lit_value(l: &Option<syn::ExprLit>) -> String {
    if l.is_none() {
        return "".to_string();
    }

    match &l.as_ref().unwrap().lit {
        syn::Lit::Str(s) => format!(r##""{}""##, s.value()),
        // syn::Lit::ByteStr(s) => format!("[{}]", &s.value().as_slice()),
        syn::Lit::Byte(s) => format!("{}", s.value()),
        syn::Lit::Char(s) => format!("{}", s.value()),
        syn::Lit::Int(s) => format!("{}", s.value()),
        syn::Lit::Float(s) => format!("{}", s.value()),
        syn::Lit::Bool(s) => format!(r##""{}""##, bool_literal(s.value)),
        // syn::Lit::Verbatim(s) => format!(r##""{}""##, s.to_string()),
        _ => "nope???".to_string(),
    }
}

fn write_comment(w: &mut dyn Write, indent: usize, comment: &str) -> std::io::Result<()> {
    writeln!(w, "{}/// {}", "\t".repeat(indent), comment)?;
    Ok(())
}

fn write_comments(w: &mut dyn Write, indent: usize, comments: &[String]) -> std::io::Result<()> {
    for c in comments {
        write_comment(w, indent, &c)?;
    }

    Ok(())
}
