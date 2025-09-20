# Annotations

## Annotating Types

Add the `#[typeshare]` attribute to any struct or enum you define to generate definitions for that type in the selected output language.

```rust
#[typeshare]
struct MyStruct {
    my_name: String,
    my_age: u32,
}

#[typeshare]
enum MyEnum {
    MyVariant,
    MyOtherVariant,
    MyNumber(u32),
}
```

## Annotation arguments

We can add arguments to the `#[typeshare]` annotation to modify the generated definitions. 

### Decorators

It can be used to add decorators like Swift protocols or Kotlin interfaces to the generated output types. For example, we can do
```rust
#[typeshare(swift = "Equatable, Codable, Comparable, Hashable")]
#[serde(tag = "type", content = "content")]
pub enum BestHockeyTeams {
    MontrealCanadiens,
    Lies(String),
}
```
and this will produce the following Swift definition:
```swift
public enum OPBestHockeyTeams2: Codable, Comparable, Equatable, Hashable {
	case montrealCanadiens
	case lies(String)

	enum CodingKeys: String, CodingKey, Codable {
		case montrealCanadiens = "MontrealCanadiens",
			lies = "Lies"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .montrealCanadiens:
				self = .montrealCanadiens
				return
			case .lies:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .lies(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPBestHockeyTeams2.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPBestHockeyTeams"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .montrealCanadiens:
			try container.encode(CodingKeys.montrealCanadiens, forKey: .type)
		case .lies(let content):
			try container.encode(CodingKeys.lies, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
```

### Serialize as Another Type

You can also use the `serialized_as` argument to tell Typeshare to treat
the serialized type as another Rust type. This is usually combined with
custom serde attributes.
```rust
/// Options that you could pick
#[typeshare(serialized_as = "String")]
#[serde(try_from = "String", into = "String")]
pub enum Options {
    /// Affirmative Response
    Yes,
    No,
    Maybe,
    /// Sends a string along
    Cool(String),
}
```
This would generate the following Kotlin code:
```kotlin
/// Options that you could pick
typealias Options = String
```

### Override Type for a Field

You can also use language-specific arguments to tell Typeshare to treat
a field as a type in a particular output language. For example,
```rust
#[typeshare]
struct MyStruct {
    #[typeshare(typescript(type = "0 | 1"))]
    oneOrZero: u8,
}
```
would generate the following Typescript code:
```typescript
export interface MyStruct {
	oneOrZero: 0 | 1;
}
```
The `type` argument is supported for all output languages, however Typescript
also supports the optional `readonly` argument (e.g. `typescript(readonly, type= "0 | 1")`)
to make the output property readonly.

### Special Note on 64 Bit Integer Types

The default behavior for 64 bit integer types when outputting TypeScript is to
panic. The reasoning behind this is that in JavaScript runtimes integers are not
sufficient to fully represent the set of all 64 bit integers, that is,
`Number.MIN_SAFE_INTEGER` and `Number.MAX_SAFE_INTEGER` are less in magnitude
than `i64::MIN` and `u64::MAX`, respectively. There are a few ways one can still
use 64 bit integer types, however, and a Typeshare attribute to override the
field type can be applied to accommodate the particular approach one chooses to
take. Here are a few examples:

**Serializing 64 bit integer fields to strings using `serde(with = ...)`**
```rust
struct MyStruct {
    #[typeshare(typescript(type = "string"))]
    #[serde(with = "my_string_serde_impl")]
    my_field: u64
}
```

**Using a third-party JSON parser that provides support for larger integer types via `bigint`**
```rust
struct MyStruct {
    #[typeshare(typescript(type = "bigint"))]
    my_field: u64
}
```

**Throwing all caution to the wind and just using `number`**
```rust
struct MyStruct {
    #[typeshare(typescript(type = "number"))]
    my_field: u64
}
```


## The `#[serde]` Attribute

Since Typeshare relies on the [`serde`](https://crates.io/crates/serde) crate for handling serialization and deserialization between Rust types and the generated foreign type definitions, we can use the annotations provided by `serde` on our Typeshare types. For example, the following Rust definition
```rust
/// This is a comment.
/// Continued lovingly here
#[typeshare]
#[serde(rename_all = "camelCase")]
pub enum Colors {
    Red = 0,
    Blue = 1,
    /// Green is a cool color
    #[serde(rename = "green-like")]
    Green = 2,
}
```
will become the following Typescript definition.
```typescript
/**
 * This is a comment.
 * Continued lovingly here
 */
export const enum Colors {
	Red = "red",
	Blue = "blue",
	/** Green is a cool color */
	Green = "green-like",
}
```

### Skipping Fields

Within a Rust type, there may be fields or variants that you want Typeshare to ignore. These can be skipped using either the `#[serde(skip)]` annotation or the `#[typeshare(skip)]` annotation. For example, this Rust type
```rust
#[typeshare]
pub struct MyStruct {
    a: i32,
    #[serde(skip)]
    b: i32,
    c: i32,
    #[typeshare(skip)]
    d: i32,
}
```
becomes the following Typescript definition.
```typescript
export interface MyStruct {
	a: number;
	c: number;
}
```
