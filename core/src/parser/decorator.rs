use proc_macro2::Ident;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::Token;
pub type Decorators = HashMap<String, Vec<Decorator>>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decorator {
    ValueEquals { key: String, value: String },
    Word(String),
}
impl Parse for Decorator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

struct LanguageDecoratorParser {
    lang: String,
    decorators: Vec<Decorator>,
}

impl Parse for LanguageDecoratorParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lang = input.parse::<Ident>()?.to_string();
        input.parse::<Token![,]>()?;
        let decorators = input.parse_terminated(Decorator::parse, Token![,])?;

        Ok(LanguageDecoratorParser {
            lang,
            decorators: decorators.into_iter().collect(),
        })
    }
}

/// Checks the struct or enum for decorators like
/// `#[typeshare_lang(swift, extends = "Codable, Equatable")]`
/// Takes a slice of `syn::Attribute`,
///
/// returns a `HashMap<language, Vec<decoration_words>>`, where `language` is `SupportedLanguage` and `decoration_words` is `String`
pub fn get_lang_decorators(attrs: &[syn::Attribute]) -> Decorators {
    // The resulting HashMap, Key is the language, and the value is a vector of decorators words that will be put onto structures
    let mut out = HashMap::new();

    for attr in attrs {
        if attr.path().is_ident("typeshare_lang") {
            let parse = attr.parse_args::<LanguageDecoratorParser>()?;
            out.insert(parse.lang, parse.decorators);
        }
    }
    //return our hashmap mapping of language -> Vec<decorators>
    out
}
