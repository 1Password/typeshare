// Based off Serde implementation: https://github.com/serde-rs/serde/blob/7950f3cdc52d4898aa4195b853cbec12d65bb091/serde_derive/src/internals/case.rs

/// Trait to rename a string using common case or character separators.
pub trait RenameExt {
    /// Convert to camelCase.
    fn to_camel_case(&self) -> String;
    /// Convert to PascalCase.
    fn to_pascal_case(&self) -> String;
    /// Convert to snake_case.
    fn to_snake_case(&self) -> String;
    /// Convert to SCREAMING_SNAKE_CASE.
    fn to_screaming_snake_case(&self) -> String;
    /// Convert to kebab-case.
    fn to_kebab_case(&self) -> String;
    /// Convert to SCREAMING-KEBAB-CASE.
    fn to_screaming_kebab_case(&self) -> String;
}

impl RenameExt for String {
    fn to_camel_case(&self) -> String {
        let pascal = self.to_pascal_case();
        pascal[..1].to_ascii_lowercase() + &pascal[1..]
    }

    fn to_pascal_case(&self) -> String {
        let mut pascal = Self::new();
        let mut capitalize = true;
        let to_lowercase = {
            // Check if string is all uppercase, such as "URL" or "TOTP". In that case, we don't want
            // to preserve the cases.
            self.to_ascii_uppercase() == *self
        };

        for ch in self.chars() {
            if ch == '_' {
                capitalize = true;
            } else if capitalize {
                pascal.push(ch.to_ascii_uppercase());
                capitalize = false;
            } else {
                pascal.push(if to_lowercase {
                    ch.to_ascii_lowercase()
                } else {
                    ch
                });
            }
        }
        pascal
    }

    fn to_snake_case(&self) -> String {
        let mut snake = Self::new();
        let is_uppercase = self.to_ascii_uppercase() == *self;
        for (i, ch) in self.char_indices() {
            if i > 0 && ch.is_uppercase() && !is_uppercase {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        }
        snake
    }

    fn to_screaming_snake_case(&self) -> String {
        self.to_snake_case().to_ascii_uppercase()
    }

    fn to_kebab_case(&self) -> String {
        self.to_snake_case().replace('_', "-")
    }

    fn to_screaming_kebab_case(&self) -> String {
        self.to_kebab_case().to_ascii_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        original: String,
        camel_case: String,
        pascal_case: String,
        snake_case: String,
        screaming_snake_case: String,
        kebab_case: String,
        screaming_kebab_case: String,
    }

    #[test]
    fn test_rename() {
        let tests = &[
            Test {
                original: "FooBar".to_string(),
                camel_case: "fooBar".to_string(),
                pascal_case: "FooBar".to_string(),
                snake_case: "foo_bar".to_string(),
                screaming_snake_case: "FOO_BAR".to_string(),
                kebab_case: "foo-bar".to_string(),
                screaming_kebab_case: "FOO-BAR".to_string(),
            },
            Test {
                original: "foo_bar".to_string(),
                camel_case: "fooBar".to_string(),
                pascal_case: "FooBar".to_string(),
                snake_case: "foo_bar".to_string(),
                screaming_snake_case: "FOO_BAR".to_string(),
                kebab_case: "foo-bar".to_string(),
                screaming_kebab_case: "FOO-BAR".to_string(),
            },
            Test {
                original: "Hello".to_string(),
                camel_case: "hello".to_string(),
                pascal_case: "Hello".to_string(),
                snake_case: "hello".to_string(),
                screaming_snake_case: "HELLO".to_string(),
                kebab_case: "hello".to_string(),
                screaming_kebab_case: "HELLO".to_string(),
            },
            Test {
                original: "Number1".to_string(),
                camel_case: "number1".to_string(),
                pascal_case: "Number1".to_string(),
                snake_case: "number1".to_string(),
                screaming_snake_case: "NUMBER1".to_string(),
                kebab_case: "number1".to_string(),
                screaming_kebab_case: "NUMBER1".to_string(),
            },
            Test {
                original: "AddressLine1".to_string(),
                camel_case: "addressLine1".to_string(),
                pascal_case: "AddressLine1".to_string(),
                snake_case: "address_line1".to_string(),
                screaming_snake_case: "ADDRESS_LINE1".to_string(),
                kebab_case: "address-line1".to_string(),
                screaming_kebab_case: "ADDRESS-LINE1".to_string(),
            },
            Test {
                original: "URL".to_string(),
                camel_case: "url".to_string(),
                pascal_case: "Url".to_string(),
                snake_case: "url".to_string(),
                screaming_snake_case: "URL".to_string(),
                kebab_case: "url".to_string(),
                screaming_kebab_case: "URL".to_string(),
            },
        ];

        for test in tests {
            assert_eq!(test.original.to_camel_case(), test.camel_case);
            assert_eq!(test.original.to_pascal_case(), test.pascal_case);
            assert_eq!(test.original.to_snake_case(), test.snake_case);
            assert_eq!(
                test.original.to_screaming_snake_case(),
                test.screaming_snake_case
            );
            assert_eq!(test.original.to_kebab_case(), test.kebab_case);
            assert_eq!(
                test.original.to_screaming_kebab_case(),
                test.screaming_kebab_case
            );
        }
    }
}
