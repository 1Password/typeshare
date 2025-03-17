# Summary

Typeshare currently supports several popular languages, but lacks support for Java. Adding support for Java would improve the state of interoperability between Rust and Java codebases. However, Java has several constraints not present in the currently supported languages. This RFC outlines a proposal for how Java support can be implemented, taking into consideration the languages idiosyncrasies.

# Basic example

This input:

```rust
/// This is a comment.
#[typeshare]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

Generates this output:

```java
/// This is a comment.
public record Person(
	/// This is another comment
	String name,
	short age,
) {}
```

# Motivation

The Rust ecosystem provides excellent tools for generating cross-language type definitions, and Typeshare is a key component for enabling seamless Rust interoperability with other languages. Currently, Typeshare supports several popular target languages like Kotlin, Scala, Swift, and Typescript. This allows multi-language projects to share types efficiently between a variety of languages.

Java is one of the most popular programming languages in use today, according to multiple surveys of the development ecosystem _[[1]](https://pypl.github.io/PYPL.html) [[2]](https://www.tiobe.com/tiobe-index/) [[3]](https://spectrum.ieee.org/top-programming-languages-2024)_. The language plays a crucial role in enterprise applications, Android development, development of backend services, and Minecraft (mod) development. Adding support for the language would improve interoperability between Java and Rust codebases, and simplify adoption of Rust components in Java projects.

Additionally, Java imposes several constraints on type generation, such as a lack of type aliasing, and strict requirements around filesystem structure (1 class per file) and naming (file names must match class names). Adding support for Java, would necessitate finding solutions for such constraints within Typeshare. In turn, this will make it simpler to both add support new languages, and support similar conventions in new and already supported languages.

# Detailed design

## Generating Structs

There are two possible ways to represent structs in Java: classes and records. These differ in several key ways, as summarized in the following table.

| | Classes | Records |
|-|-|-|
| Purpose | Storing data | Associating data with behavior |
| Immutable | no | yes |
| Minimum Java Version | - | 15 |
| Verbosity | high | low |

To highlight these differences, the following code snippets _[[4]](https://docs.oracle.com/en/java/javase/15/language/records.html)_ compare a simple Java class and the equivalent record.

```java
public final class Rectangle {
    private final double length;
    private final double width;

    public Rectangle(double length, double width) {
        this.length = length;
        this.width = width;
    }

    double length() { return this.length; }
    double width()  { return this.width; }

    // Implementation of equals() and hashCode(), which specify
    // that two record objects are equal if they
    // are of the same type and contain equal field values.
    public boolean equals...
    public int hashCode...

    // An implementation of toString() that returns a string
    // representation of all the record class's fields,
    // including their names.
    public String toString() {...}
}
```

```java
record Rectangle(double length, double width) { }
```

Generating records would be simpler to implement, as less (complex) code needs to be generated in most cases. Additionally, certain functionality, is provided "out of the box" by records which needs to be manually implemented in classes. This includes sane defaults for equality checks and `toString` implementations. Records are also easier to code review, as significantly less boilerplate is required.

However, language support could be a deal breaker for some projects which have not yet upgraded to Java 15. Furthermore, records impose certain limitations, such as immutability, which may not be desirable in some cases.

**Proposed Solution**

In order to service a wide range of projects and preferences, typeshare should generate records by default, but allow generating classes via the following CLI flag.

```sh
$ typeshare --help
...
      --java-prefer-classes        Prefer generating Java classes over records
...
```

## Handling New Type Structs and Type Aliases

Currently, new type structs _[[5]](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)_ are interpreted as type aliases when generating types. However, since Java does not support type aliasing, it is not possible to take this approach for Java. Some solutions to this problem are discussed, using the following code snippet as an example. Note that this discussion also applies to true Rust type aliases _[[6]](https://doc.rust-lang.org/reference/items/type-aliases.html)_, which are handled similarly to new type structs by Typeshare.

```rust
#[typeshare]
struct PersonName(String);

#[typeshare]
struct Person {
  name: PersonName,
}
```

**Just Crash**

Probably the simplest option is to return an "unsupported" error in cases where a type alias would normally be generated. Although simple, this can require changes to the structure of existing Rust code to accommodate Typeshare. This is not ideal, particularly because new type structs are a common and useful pattern in idiomatic Rust code.

**Type Alias Resolution**

In most cases it is possible to resolve a type aliases and or new type structs to an equivalent inline type. Doing so means that writing type aliases can be skipped in generated code where the language does not support type aliases.

Resolving type aliases to an inline type could result in the following generation for the above example.

```java
record Person(
  /* PersonName was resolved to type String */ String name
) { }
```

In some cases, a type alias may be generic. For example:

```rust
#[typeshare]
struct PersonName<T: AsRef<str>>(T);

#[typeshare]
struct Person<T: AsRef<str>> {
    name: PersonName<T>,
}
```

This necessitates correct resolution of the type arguments, yielding generated Java code similar to that in the following snippet.

```java
record Person<T>(
  /* PersonName was resolved to type T */ T name
) { }
```

In order to handle nested aliases, the type resolver would need to be capable of recursively resolving type aliases. Such a scenario is demonstrated in the following snippet.

```rust
#[typeshare]
struct Person<T>(T);

#[typeshare]
type People<T> = HashSet<Person<T>>;

#[typeshare]
struct Community<T: AsRef<str>> {
	people: People<T>,
}
```

This would generate Java types similar to those in the following snippet.

```java
record Community<T>(
	/* People was resolved to type T */ T people
) { }
```

> [!IMPORTANT]
> Ideally, it would also be possible to resolve default types for generics, as in the following example:
> 
> ```rust
> type PersonName<T = String> = T;
> 
> struct Person {
> 	name: PersonName,
> }
> ```
> 
> However, the `RustItem` enum does not currently capture the default values for generic type parameters, which leads to invalid code generation for both Java and existing languages where default values type parameters are used.
> 
> Fixing this issue is **out of scope** for this RFC.

In order to implement type alias resolution **within the same file**, the `ParsedData` struct (passed as `data` to `Language::generate_types`) needs to be available to several methods within the `Language` trait, for type resolution. This is already possible by without changes to the `Language` trait, for example by adding a optional `context` field to the language struct. This can then be set in the `Language::generate_types`, as shown in the following snippet.

```rust
struct JavaContext {
	parsed_data: ParsedData,
}

#[derive(Default)]
pub struct Java {
	pub context: Option<JavaContext>,
	// ...
}

impl Language for Java {
    fn generate_types(
        &mut self,
        writable: &mut dyn Write,
        all_types: &CrateTypes,
        data: ParsedData,
    ) -> std::io::Result<()> {
        self.context = Some(JavaContext { parsed_data: data });

        // ...
    }
    
    // ...
    
}
```

However, this is a slightly inelegant solution for the following reasons:

1) The `context` field is necessarily public, even though it is only used privately. This is because otherwise it would not be possible to instantiate the language from outside the module.
2) Making `JavaContext` private triggers the `private_interfaces` Clippy lint, but JavaContext is never (and should never be) referenced outside of the module.
3) Because `JavaContext` is necessarily optional, it must either be accessed using `unwrap` or code must handle the unreachable `None` condition.
4) Because much of the contextual data is only ever provided to `Language::generate_types`, this would necessarily need to be overridden, which could lead to unnecessary duplication of the default implementation.

Instead, it may be beneficial to extend the `Language` trait with an associated type, and implement a method to instantiate it, as follows:

```rust
pub trait Language {
	type Context;

	fn make_context(all_types: CrateTypes, data: ParsedData) -> Self::Context;

	// ...

}
```

This can be called ahead of `Language::generate_types`, and the resulting context can be passed to all other methods on the `Language` trait. Note that associated type defaults are currently unstable [[7]](https://github.com/rust-lang/rust/issues/29661), otherwise it may be possible to provide a default implementation like this:

```rust
pub trait Language {
	type Context = DefaultLanguageContext;

	fn make_context(all_types: CrateTypes, data: ParsedData) -> Self::Context {
		DefaultLanguageContext {
			all_types,
			data,
		}
	}

	// ...

}
```

Note that this does not in itself allow the language implementation to resolve all type aliases, as type aliases can be shared between modules. In order to resolve type aliases imported from other modules, it would be necessary to make the entire module graph available in the context.

**Class Aliases**

In some circumstances, type aliases can be emulated by extending existing classes. For the example above, the following code could be generated:

```java
class PersonName extends String { }

record Person(
  PersonName name
) { }
```

However, this doesn't work for several reasons:

1. Final classes (like `String`) cannot be extended
2. Java primitive types like `integer` cannot be extended
3. Records cannot be extended

**Wrapped Types**

Instead of generating type aliases, the aliased type could simply be wrapped by a class or record.

```java
record PersonName(
	String inner
) { }

record Person(
  PersonName name
) { }
```

However, this likely would result in incorrect serialisation and deserialisation of data, as it modifies the structure. Since Java doesn't have a standardised mechanism to annotate how a class should be serialised or deserialised, it also wouldn't be possible to resolve this issue in a way which works with all serialisation libraries.

**Proposed Solution**

Type alias resolution is likely the best solution for end users, as it does not require them to change the structure of their Rust code to facilitate the generation of Java types. However, to fully support this without edge cases would require non-trivial refactoring of existing language agnostic code, and requires minor modifications to all language implementations.

## Generating Enums

Rust enums come in two varieties: simple and algebraic. Algebraic enums (often referred to as algebraic data types _[[8]](https://en.wikipedia.org/wiki/Algebraic_data_type)_) are those where data is associated with one or more variants, while unit enums are fully represented by just their variant.

Although, Java does support algebraic data types through a combination of records and sealed interfaces [[9]](https://openjdk.org/projects/amber/design-notes/records-and-sealed-classes), Java enums are not algebraic.

Therefore, we need to implement separate methods for generating algebraic and unit enums.
### Generating Unit Enums

In most cases, unit enums can be trivially generated, as shown in the below snippets.

```rust
#[typeshare]
enum Color {
	Red,
	Blue,
	Green,
}
```

```java
enum Colors {
	Red,
	Blue,
	Green
}
```

In this simple case, the generated enum is syntactically identical to the source enum, less a trailing comma.

In some cases, users may rename enum variants. The renamed variant may or may not be a valid Java identifier [[10]](https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-3.8). In cases where it is a valid identifier, it can be transformed directly, as shown in the following snippets.

```rust
#[typeshare]
#[serde(rename_all = "camelCase")]
enum Color {
	Red,
	Blue,
	Green,
}
```

```java
enum Color {
	red,
	blue,
	green
}
```

However, since Java has no language level primitives for (de)serialisation, there is no way to "correctly" represent this. Or, to put it another way, the correct representation of the rename depends on the library used for (de)serialisation by the end user.

**Proposed Solutuion**

Unit enums should be directly translated to Java syntax. Where renames are applied, the enum variants should be renamed **only** if the renamed value is a valid Java identifier, in order to avoid syntax errors. End users can manually specify the required Java annotations on enum variants for their preferred (de)serialisation library using the `typeshare` macro (see Handling Renames).

### Generating Algebraic Enums

Algebraic enums can be represented using a combination of Java sealed interfaces and records, as shown in the following snippets.

```rust
#[typeshare]
enum AlgebraicEnum<T> {
	A,
	B(String),
	C {
		inner: T,
	},
}
```

```java
sealed interface StringNumberOrOtherValue<T>
	permits AlgebraicEnum.A, AlgebraicEnum.B, AlgebraicEnum.C 

	record A<T>() implements AlgebraicEnum<T> {}
	record B<T>(String value) implements AlgebraicEnum<T> {}
	record C<T>(T inner) implements AlgebraicEnum<T> {}
}
```

Similarly to unit enums, renames are only supported when they correspond to valid Java identifiers.

> [!IMPORTANT]
> When generating algebraic enums, only **untagged** representations are supported by default. This is because the implementation of (de)serialisation logic for other tag types varies by library.
> 
> For example, when using Gson, other tag types can be implemented as using `TypeAdapter` classes _[[11]](https://www.javadoc.io/doc/com.google.code.gson/gson/2.8.1/com/google/gson/TypeAdapter.html)_.
> 
> It is feasible that in future, default implementations for type adapters could be optionally generated by passing a CLI flag, but this is out of scope for this RFC.

## Handling Renames

Since renames may or may not be valid Java identifiers, it may or may not be possible to handle them directly. In cases where renames are invalid Java identifiers, Typeshare must provide escape hatches to manually specify renames.

For example, to facilitate renaming enum variants to arbitrary values, it should be possible to specify arbitrary Java annotations for enum variants. This is shown in the following snippets.

```rust
#[typeshare]
enum Color {
	Red,
	Blue,
	#[serde(rename = "green-like")]
	#[typeshare(java = "SerializedName('green-like')")]
	Green,
}
```

```java
enum Colors {
	Red,
	Blue,
	@SerializedName('green-like')
	Green
}
```

> [!IMPORTANT]
> Currently, it is not possible to pass decorators or annotations to enum variants in this manner. Therefore, some language agnostic changes need to be implemented to support this feature.

## Handling Special Rust Types

The below table outlines the mapping of "special Rust types" to Java types.

| Rust Type       | Java Type [[12]](https://docs.oracle.com/javase/specs/jls/se23/html/jls-4.html#jls-IntegralType) | Comment                                                                                    |
| --------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| `Vec<T>`        | `java.util.ArrayList<T>`                                                                         | -                                                                                          |
| `[T; N]`        | `T[]`                                                                                            | There is no fixed length array type in Java.                                               |
| `&[T]`          | `T[]`                                                                                            | -                                                                                          |
| `HashMap<K, V>` | `java.util.HashMap<K, V>`                                                                        | -                                                                                          |
| `Unit`          | `Void`                                                                                           | -                                                                                          |
| `String`        | `String`                                                                                         | -                                                                                          |
| `i8`            | `byte`                                                                                           | -                                                                                          |
| `i16`           | `short`                                                                                          | -                                                                                          |
| `ISize`, `i32`  | `int`                                                                                            | -                                                                                          |
| `I54`, `i64`    | `long`                                                                                           | -                                                                                          |
| `u8`            | `short`                                                                                          | `byte` in Java is signed, so we need to use `short` to represent all possible values.      |
| `u16`           | `int`                                                                                            | `short` in Java is signed, so we need to use `int` to represent all possible values.       |
| `u32`           | `long`                                                                                           | `int` in Java is signed, so we need to use `long` to represent all possible values.        |
| `u64`           | `java.math.BigInteger`                                                                           | `long` in Java is signed, so we need to use `BigInteger` to represent all possible values. |
| `bool`          | `boolean`                                                                                        | -                                                                                          |
| `f32`           | `float`                                                                                          | -                                                                                          |
| `f64`           | `double`                                                                                         | -                                                                                          |
| `DateTime`      | -                                                                                                | Unsupported type.                                                                          |

> [!IMPORTANT]
> For simplicity of implementation, Java utility types such as `HashMap` will be referenced by their fully qualified name (e.g. `java.util.HashMap`). This avoids having to add imports to the top of the file, based on the Rust special types used within it.
## Handling Option Types

In Java, all types are nullable by default. It is idiomatic to annotate non-nullable types with an `@NotNull` annotation, but this annotation is not provided by the standard library. As such, `Option<T>` types are generated identically to `T` types.

```rust
struct MaybePerson {
	person: Option<Person>,
}
```

```java
record MaybePerson(
	Person person
) { }
```

## Single File per Class & File Naming

In Java, each file can contain only one public class, and file names must match class names. Currently, Typeshare generates output files one to one with source files. This can result in the generation of invalid Java files. Requiring consumers to refactor their Rust code based on Java specific constraints would be undesirable. Therefore, several solutions have been evaluated.

**Language Agnostic CLI Flag**

It would be possible to implement a language agnostic CLI flag to generate one output file per struct, enum, etc. However, this is not scalable, as different languages may have different constraints and conventions for file generation and naming.

**Produce Invalid Code / Error**

As already discussed, producing invalid code (or erroring) would force consumers of Typeshare to refactor their Rust code based on Java concerns. This strategy is not scalable for Typeshare because future languages may have conflicting constraints, making it impossible to support both with a single codebase.

**Move File Generation to Language Trait**

File generation is currently the responsibility of the `write_multiple_files` function. This could be moved to the `Language` trait, with the default implementation preserving the current behaviour.

**Proposed Solution**

File generation should be moved to the `Lanugage` trait.

# Drawbacks

- Requires refactoring existing, and (mostly) working code
- Maintenance burden of Java language may not be worth it if there is insufficient demand for generating Java types using Typeshare

# Alternatives

Although several solutions exist for producing Java FFI bindings from Rust _[[13]](https://github.com/MaulingMonkey/jni-bindgen) [[14]](https://github.com/astonbitecode/j4rs)_, there are currently no existing projects who's goals are aligned with those of Typeshare, and which support Java in addition to the other languages already supported by Typeshare.

# Adoption strategy

Initially, it may be sensible to release this feature behind a feature flag, similar to the Go and Python implementations. Once stable, it can be unflagged, making it available to all Typeshare users.

# Documentation

End user documentation will be similar to that for other languages. However, if the internal refactors are implemented as proposed, it may be necessary to update developer documentation. Comments on new trait functions should be added, explaining their usage, and additional markdown documentation should be produced to explain the concepts behind these, as well as when to (not) use them in the implementation of a new language.

# Unresolved questions

This is a first draft. All aspects are open for discussion and feedback.

Specifically, feedback is welcome on the following open questions:

1. Design of field and variant level decorators
2. Design of single file per class generation
3. Possible introduction of an associated type for context
4. Guidance on how to handle type aliases and possible resolution to inline types
5. Records vs classes
6. Which parts of this RFC need to be implemented for an MVP solution?
7. What is the bar that needs to be met before the feature can be considered stable?
