use crate::parsed_types::{Decorator, DecoratorsMap};
use proc_macro2::{Ident, TokenStream};

use syn::parse::{Parse, ParseStream};
use syn::{custom_keyword, Meta, Token};

impl Parse for Decorator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse::<syn::LitStr>()?.value();
            return Ok(Decorator::LangType(value));
        }
        let ident = input.parse::<Ident>()?.to_string();
        return if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let value = input.parse::<syn::LitStr>()?.value();

            Ok(Decorator::ValueEquals { key: ident, value })
        } else {
            Ok(Decorator::Word(ident))
        };
    }
}

enum LanguageDecoratorParser {
    Found {
        lang: String,
        decorators: Vec<Decorator>,
    },
    NotFound,
}
custom_keyword!(lang);
impl Parse for LanguageDecoratorParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(lang) {
            let _ = input.parse::<TokenStream>();
            return Ok(LanguageDecoratorParser::NotFound);
        }
        let _ = input.parse::<lang>()?;
        let _ = input.parse::<Token![=]>()?;
        let lang = input.parse::<Ident>()?.to_string();
        let content;
        let _ = syn::parenthesized!(content in input);
        let decorators = content.parse_terminated(Decorator::parse, Token![,])?;

        Ok(LanguageDecoratorParser::Found {
            lang,
            decorators: decorators.into_iter().collect(),
        })
    }
}

/// Checks the struct or enum for decorators like
/// `#[typeshare(lang = swift(extends = "Codable, Equatable"))]`
/// Takes a slice of `syn::Attribute`,
///
/// returns a `HashMap<language, Vec<decoration_words>>`, where `language` is `SupportedLanguage` and `decoration_words` is `String`
pub fn get_lang_decorators(attrs: &[syn::Attribute]) -> Result<DecoratorsMap, syn::Error> {
    // The resulting HashMap, Key is the language, and the value is a vector of decorators words that will be put onto structures
    let mut out = DecoratorsMap::default();

    for attr in attrs {
        if attr.path().is_ident("typeshare") {
            if let Meta::List(_) = &attr.meta {
                let parse = attr.parse_args::<LanguageDecoratorParser>()?;
                if let LanguageDecoratorParser::Found { lang, decorators } = parse {
                    out.insert(lang, decorators);
                }
            }
        }
    }
    //return our hashmap mapping of language -> Vec<decorators>
    Ok(out)
}
