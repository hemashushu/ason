# ASON

_ASON_ is a data serialization format that evolved from JSON, featuring strong numeric typing and native support for enumeration types. With excellent readability and maintainability, ASON is well-suited for configuration files, data transfer, and data storage.

**Table of Content**

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->

- [1. ASON Example](#1-ason-example)
- [2. Comparison of Common Data Serialization Formats](#2-comparison-of-common-data-serialization-formats)
- [3. What improvements does ASON bring over JSON?](#3-what-improvements-does-ason-bring-over-json)
- [4 Library and APIs](#4-library-and-apis)
  - [4.1 Serialization and Deserialization](#41-serialization-and-deserialization)
  - [4.2 Parser and Writer](#42-parser-and-writer)
  - [4.3 Token Stream Reader and Writer](#43-token-stream-reader-and-writer)
- [5 ASON Quick Reference](#5-ason-quick-reference)
  - [5.1 Primitive Values](#51-primitive-values)
    - [5.1.1 Digit Separators](#511-digit-separators)
    - [5.1.2 Explicit Numeric Types](#512-explicit-numeric-types)
    - [5.1.3 Hexadecimal, Octal and Binary Integers](#513-hexadecimal-octal-and-binary-integers)
    - [5.1.4 Hexadecimal Floating-Point Numbers](#514-hexadecimal-floating-point-numbers)
    - [5.1.5 Special Floating-Point Numbers](#515-special-floating-point-numbers)
    - [5.1.6 String Presentation](#516-string-presentation)
  - [5.2 Compound Values](#52-compound-values)
    - [5.2.1 Lists](#521-lists)
    - [5.2.2 Tuples](#522-tuples)
    - [5.2.3 Objects](#523-objects)
    - [5.2.4 Named Lists](#524-named-lists)
    - [5.2.5 Enumerations](#525-enumerations)
    - [5.2.6 Type of Compound Values](#526-type-of-compound-values)
  - [5.3 Comments](#53-comments)
  - [5.4 Documents](#54-documents)
- [6 Mapping between Rust Data Types and ASON Types](#6-mapping-between-rust-data-types-and-ason-types)
  - [6.1 Structs](#61-structs)
  - [6.2 Vecs](#62-vecs)
  - [6.3 HashMaps](#63-hashmaps)
  - [6.4 Tuples](#64-tuples)
  - [6.5 Enums](#65-enums)
  - [6.6 Other Data Types](#66-other-data-types)
  - [6.7 Default Values](#67-default-values)
- [7 ASON Specification](#7-ason-specification)
- [8 Linking](#8-linking)
- [9 License](#9-license)

<!-- /code_chunk_output -->

## 1. ASON Example

An example of ASON document:

```json5
{
    string: "Hello World 🍀"
    raw_string: r"[a-z]\d+"
    integer_number: 123
    floating_point_number: 3.14
    number_with_explicit_type: 255_u8
    hexadecimal_integer: 0x2B
    hexadecimal_floating_point_number: 0x1.921FB6p+1
    octal_integer: 0o755
    binary_integer: 0b0101_1000
    boolean: true
    datetime: d"2023-03-24 12:30:00+08:00"
    bytedata: h"68 65 6c 6c 6f 0a 00"
    list: [11, 13, 17, 19]
    named_list: [
        "foo": "The quick brown fox jumps over the lazy dog"
        "bar": "My very educated mother just served us nine pizzas"
    ]
    tuple: (1, "Hippo", true)
    object: {
        id: 123
        name: "HttpClient"
        version: "1.0.1"
    }
    variant_without_value: Option::None
    variant_with_value: Option::Some(123)
    variant_with_tuple_like_value: Color::RGB(255, 127, 63)
    variant_with_object_like_value: Shape::Rect{
        width: 200
        height: 100
    }
}
```

## 2. Comparison of Common Data Serialization Formats

There are many solid data serialization formats available today, such as JSON, YAML, TOML and XML. They are all designed to be readble and writable by both humans and machines.

The differences between these formats are minor for small datasets, but become more pronounced as datasets grow or structures become more complex. For example, YAML's indentation-based syntax can cause errors in editing large documents, and TOML's limited support for complex structures can make representing hierarchical data cumbersome. JSON, by contrast, is designed to stay simple, consistent, and expressive at any scale.

For developers, JSON offers additional advantages:

- You don't have to learn an entirely new syntax: JSON closely resembles JavaScript object literals.
- Implementing a parser is straightforward, which helps ensure longevity and adaptability across evolving software ecosystems.

## 3. What improvements does ASON bring over JSON?

JSON has a simple syntax and has been around for decades, but it struggles to meet diverse modern needs. Many JSON variants have emerged to address its limitations—such as JSONC (which adds comments) and JSON5 (which allows trailing commas and unquoted object keys). However, these variants still cannot represent data accurately due to JSON has a limited type system and lacks fine-grained numeric and domain-specific data types. ASON takes a significant step forward based on JSON with the following improvements:

- **Explicit Numeric Types:** ASON numbers can be explicitly typed (e.g., `u8`, `i32`, `f32`, `f64`) ensuring more precise and rigirous data representation. Additionally, integers can be represented in hexadecimal, octal, and binary formats.
- **New Data Types:** New data types such as `Char`, `DateTime`, and `HexadecimalByteData` to better represent common data types.
- **More string formats:** "Multi-line strings", "Concatenate strings", "Raw strings", and "Auto-trimmed strings" are added to enhance string representation.
- **Separate List and Tuple:** ASON distinguishes between `List` (homogeneous elements) and `Tuple` (heterogeneous elements), enhancing data structure clarity.
- **Separate Named-List and Object:** ASON introduces `Named-List` (also called `Map`) alongside `Object` (also called `Struct`), enhancing data structure clarity in further.
- **Native Enumerations Support:** ASON natively supports enumerations types (also known as _Algebraic types_ or _Variants_). This enables seamless serialization of complex data structures from high-level programming languages.
- **Eliminating the Null Value:** ASON uses the `Option` enumeration to represent optional values, eliminating the error-prone `null` value.
- **Simple and Consistent:** ASON supports comments, unquoted object field names, trailing commas, and whitespace-separated elements (in addition to commas). These features enhance writing fluency.

In addition to the text format, ASON provides a binary format called _ASONB_ (ASON Binary) for efficient data storage and transmission. ASONB supports incremental storage, memory-mapped file access, and fast random access.

While ASON is designed to resemble JSON, making it easy for JSON users to learn and adopt, it is not compatible with JSON, but conversion between ASON and JSON is straightforward, and implementing an ASON parser is also simple.

## 4 Library and APIs

The Rust [ason](https://github.com/hemashushu/ason) library provides AST (Abstract Syntax Tree) and Token level ASON access, and [serde_ason](https://github.com/hemashushu/serde_ason) provides [serde](https://github.com/serde-rs/serde) based for serialization and deserialization.

In general, it is recommended to use the serde API since it is simple enough to meet most needs.

### 4.1 Serialization and Deserialization

Consider the following ASON document:

```json5
{
    name: "foo"
    version: "0.1.0"
    dependencies: [
        "random@1.0.1"
        "regex@2.0.0"
    ]
}
```

This document consists of an object and a list: the object with `name`, `version` and `dependencies` fields, and the list with string as elements. We can create a Rust struct corresponding to these data:

```rust
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
}
```

The struct needs to be annotated with a `derive` attribute, in which `Serialize` and `Deserialize` are traits provided by the _serde_ serialization framework.

The following code shows how to use the serde API `ason::serde::from_str` for deserializing the ASON document into a Rust struct instance:

```rust
let text = "..."; // The above ASON document
let package = from_str::<Package>(text).unwrap(); // Now you get a `Package` struct instance
```

You can serialize a Rust struct instance to string with `ason::serde::to_string` function:

```rust
let package = Package{
    name: String::new("foo"),
    version: String::new("0.1.0"),
    dependencies: vec![
        String::new("random@1.0.1"),
        String::new("regex@2.0.0"),
    ],
};
let text = to_string(&package);
// The `text` should be resemble the above ASON document
```

### 4.2 Parser and Writer

You can parse the ASON document into AST object using the `ason::parser::parse_from_str` function:

```rust
let text = "..."; // The above ASON document
let node = parse_from_str(text).unwrap(); // Now you get an AST object of type `AsonNode`

// Let's verify the structure of the AST object
assert_eq!(
    node,
    AsonNode::Object(vec![
        KeyValuePair {
            key: String::from("name"),
            value: Box::new(AsonNode::String(String::from("foo")))
        },
        KeyValuePair {
            key: String::from("version"),
            value: Box::new(AsonNode::String(String::from("0.1.0")))
        },
        KeyValuePair {
            key: String::from("dependencies"),
            value: Box::new(AsonNode::List(vec![
                AsonNode::String(String::from("random@1.0.1")),
                AsonNode::String(String::from("regex@2.0.0"))
            ]))
        }
    ])
);
```

You can also turn the AST object into a string using the `ason::writer::write_to_string` function:

```rust
let text = write_to_string(&node);
// The `text` should be resemble the above ASON document
```

Since AST object lacks some information such as comments, whitespace, the original string format (e.g., multi-line string, raw string, etc.), and the original numeric types (e.g., hexadecimal, octal, binary), so the output text may not be exactly the same as the input text, do not use the writer for formatting ASON documents.

### 4.3 Token Stream Reader and Writer

ASON Rust library also provides a token stream reader and writer for even more low-level access to ASON documents.

Consider the following ASON document:

```json5
{
    id: 123
}
```

The token stream reader can be used to read the document token by token:

```rust
let text = "..."; // The above ASON document
let mut reader = stream_from_str(text);

// The first token should be the opening brace `{`.
// `reader.next()` returns `Option<Result<Token, AsonError>>`,
// so we need to unwrap the `Option` first.
let first_result = reader.next().unwrap();
assert!(first_result.is_ok());

let first_token = first_result.unwrap();
assert_eq!(first_token, Token::OpeningBrace);

// The next token should be the identifier `id`.
assert_eq!(
    reader.next().unwrap().unwrap(),
    Token::Identifier("id".to_string())
);

// The next token should be the colon `:`.
assert_eq!(reader.next().unwrap().unwrap(), Token::Colon);

// The next token should be the number `123`, which is of type `i32` by default.
assert_eq!(
    reader.next().unwrap().unwrap(),
    Token::Number(NumberToken::I32(123))
);

// The last token should be the closing brace `}`.
assert_eq!(reader.next().unwrap().unwrap(), Token::ClosingBrace);

// There should be no more tokens.
assert!(reader.next().is_none());
```

Token stream reader does not verify the syntax of the document while it checks the validity of each token, it is generally used for syntax highlighting and linting, or reading large documents without loading the entire document into memory.

There is also a token stream writer for writing tokens into a stream, which is typically used for generating ASON documents incrementally.

```rust
let mut output = Vec::new(); // Or other types of output stream

let mut writer = TokenStreamWriter::new(&mut output);
writer.print_token(&Token::OpeningBrace)?;
writer.print_token(&Token::Identifier("id".to_owned()))?;
writer.print_token(&Token::Colon)?;
writer.print_space()?;
writer.print_token(&Token::Number(NumberToken::I32(123)))?;
writer.print_space()?;
writer.print_token(&Token::Identifier("name".to_owned()))?;
writer.print_token(&Token::Colon)?;
writer.print_space()?;
writer.print_token(&Token::String("Alice".to_owned()))?;
writer.print_token(&Token::ClosingBrace)?;

// Verify the output
let text = String::from_utf8(output).unwrap();
assert_eq!(text, "\n{    id: 123 name: \"Alice\"\n}");
```

Similar to the token stream reader, the token stream writer does not verify the token sequence, it can write any string (including comments and whitespace) as you want, it is just a thin wrapper around the output stream.

## 5 ASON Quick Reference

ASON is composed of values, there are two kinds of values: primitive and compound. Primitive values are basic data types like integers, strings, booleans. Compound values are structures made up of multiple values (includes primitive and other compound values), such as lists and objects.

### 5.1 Primitive Values

ASON supports the following primitive value types:

**Numeric Values:**

- Integers: `123`, `+456`, `-789`
- Floating-point numbers: `3.142`, `+1.414`, `-1.732`
- Floating-point with exponent notation: `2.998e10`, `6.674e-11`
- Special floating-point values: `NaN`, `Inf`, `+Inf`, `-Inf`
- Hexadecimal integers: `0x41`, `+0x51`, `-0x61`, `0x71`
- Hexadecimal floating-point: `0x1.4p3`, `0x1.921f_b6p1`
- Octal integers: `0o755`, `+0o600`, `-0o500`
- Binary integers: `0b1100`, `+0b1010`, `-0b1010_0100`

**Text Values:**

- Characters: `'a'`, `'文'`, `'😊'`
- Escape sequences: `'\r'`, `'\n'`, `'\t'`, `'\\'`
- Unicode escapes: `'\u{2d}'`, `'\u{6587}'`
- Strings: `"abc文字😊"`, `"foo\nbar"`
- Raw strings: `r"[a-z]+\d+"`, `r#"<\w+\s(\w+="[^"]+")*>"#`

**Boolean, Date and Binary Values:**

- Booleans: `true`, `false`
- Date and time: `d"2024-03-16"`, `d"2024-03-16 16:30:50"`, `d"2024-03-16T16:30:50Z"`, `d"2024-03-16T16:30:50+08:00"`
- Hexadecimal byte data: `h"11 13 17 19"`

#### 5.1.1 Digit Separators

To improve readability, underscores can be inserted between digits in any numeric literal. For example: `123_456_789`, `6.626_070_e-34`

#### 5.1.2 Explicit Numeric Types

Numbers can be explicitly typed by appending a type suffix. For example: `65u8`, `3.14f32`

ASON supports the following numeric types: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`.

> If no explicit type is specified, integers default to `i32` and floating-point numbers default to `f64`.

Underscores may also appear between the number and its type suffix for readability: `933_199_u32`, `6.626e-34_f32`

#### 5.1.3 Hexadecimal, Octal and Binary Integers

Beyond decimal notation, ASON supports three additional integer formats:

- **Hexadecimal**: prefix with `0x` (e.g., `0xFF`)
- **Octal**: prefix with `0o` (e.g., `0o755`)
- **Binary**: prefix with `0b` (e.g., `0b1010`)

Type prefixes are case-insensitive. Note that leading zeros are not permitted for decimal integers (`0123` is invalid), but the single digit `0` is valid.

#### 5.1.4 Hexadecimal Floating-Point Numbers

Appending a type suffix directly to a hexadecimal integer creates ambiguity because `f` is a valid hexadecimal digit. For example, `0x21_f32` would be parsed as the hexadecimal integer `0x21f32` rather than a typed floating-point number.

To represent hexadecimal floating-point literals, use the P notation format: `0x1.921f_b6p1`, `0x1.4p3`. For details, see [P notation](https://en.wikipedia.org/wiki/Hexadecimal#Exponential_notation) or [Hexadecimal floating point literals](https://www.ibm.com/docs/en/xl-c-and-cpp-aix/16.1.0?topic=literals-floating-point).

#### 5.1.5 Special Floating-Point Numbers

ASON supports special floating-point values: `NaN`, `Inf`, `+Inf`, `-Inf`. These represent results of certain mathematical operations (such as division by zero or square root of a negative number) and are not valid in JSON.

Important notes:

- Do not confuse these with strings: `"NaN"` is a string, while `NaN` (unquoted) is a special floating-point value.
- Leading signs are not allowed for `NaN` (`-NaN` and `+NaN` are invalid).
- Type suffixes are optional but allowed: `NaN_f32`, `+Inf_f32` (underscore is mandatory when a suffix is used). The default type is `f64`.

#### 5.1.6 String Presentation

Strings in ASON can be represented in multiple ways: normal strings, raw strings, multi-line strings, concatenated strings, and auto-trimmed strings.

##### 5.1.6.1 Multi-Line Strings

Strings in ASON are enclosed in double quotation marks (`"`). A string ends when the closing quotation mark is reached, allowing strings to span multiple lines.

For example:

```json5
{
    multiline_string: "Planets in the Solar System:
        1. Mercury
        2. Venus
        3. Earth
        4. Mars
        5. Jupiter
        6. Saturn
        7. Uranus
        8. Neptune"
}
```

To include a double quotation mark (`"`) in a string, escape it with a backslash (`\"`). Similarly, to include a backslash (`\`), escape it with another backslash (`\\`).

In this form, all whitespace (including line breaks, tabs, and spaces) becomes part of the string content. For example, the string above is equivalent to:

```text
"Planets in the Solar System:
        1. Mercury
        2. Venus
        3. Earth
        4. Mars
        5. Jupiter
        6. Saturn
        7. Uranus
        8. Neptune"
```

Note that all leading spaces are preserved.

###### 5.1.6.2 Concatenated Strings

To improve readability, ASON allows splitting a long string across multiple lines by adding a backslash (`\`) at the end of each line. The subsequent text will be concatenated with leading whitespace removed. For example:

```json5
{
    long_string: "My very educated \
        mother just served \
        us nine pizzas"
}
```

This is equivalent to `"My very educated mother just served us nine pizzas"`. This form is especially useful for representing paragraphs or long sentences without introducing unwanted line breaks.

###### 5.1.6.3 Auto-Trimmed Strings

Auto-trimmed strings provide another way to represent multi-line strings. They automatically trim the same amount of leading whitespace from each line. For example:

```json5
{
    auto_trimmed_string: """
           Planets and Satellites
        1. The Earth
           - The Moon
        2. Saturn
           - Titan
             Titan is the largest moon of Saturn.
           - Enceladus
        3. Jupiter
           - Io
             Io is one of the four Galilean moons of the planet Jupiter.
           - Europa
        """
}
```

The leading whitespace serves only for indentation and readability; it is not part of the string content.

In this example, lines have 8, 11, or 13 leading spaces. Since the minimum is 8 spaces, exactly 8 leading spaces are removed from each line. The resulting string is:

```text
   Planets and Satellites
1. The Earth
   - The Moon
2. Saturn
   - Titan
     Titan is the largest moon of Saturn.
   - Enceladus
3. Jupiter
   - Io
     Io is one of the four Galilean moons of the planet Jupiter.
   - Europa
```

When writing auto-trimmed strings, follow these rules:

- The opening `"""` must be immediately followed by a line break.
- The closing `"""` must start on a new line; leading spaces are allowed, but they are not counted.
- In blank lines, all spaces are not counted.

In short, the syntax of auto-trimmed strings is:

`"""\n...\n"""`

Where `...` represents the content lines (the two line breaks are mandatory, and they are not part of the string content).

Another example:

```json5
[
  """
    Hello
  """, """
    Earth
      &
    Mars
  """
]
```

The two strings are equivalent to `"Hello"` and `"Earth\n  &\nMars"`.

### 5.2 Compound Values

ASON supports the following compound value types: Lists, Tuples, Objects, Named Lists and Enumerations.

#### 5.2.1 Lists

A List is a collection of values, for example, this is a List of integers:

```json5
[11, 13, 17, 19]
```

And this is a List of strings:

```json5
["Alice", "Bob", "Carol", "Dan"]
```

All the elements in a List must be of the same type. For example, the following List is invalid because it contains both integers and strings:

```json5
[11, 13, "Alice", "Bob"]    // invalid list
```

##### 5.2.1.1 Element Separators

The elements in a List can also be written on separate lines, with optional commas at the end of each line:

```json5
[
    "Alice",
    "Bob",
    "Carol",
    "Dan",  // Tail comma is allowed.
]
```

A comma following the last element (tail comma) is allowed, this feature is primarily intended to make it easy to reorder elements when editing multi-line lists.

Commas in ASON are used as separators for readability, but they are not mandatory. Therefore, the following List is valid and is equivalent to the above List:

```json5
[
    "Alice"  // Commas can be omitted.
    "Bob"
    "Carol"
    "Dan"
]
```

Whitespace (such as spaces, tabs, and line breaks) can also be used as separators, so the following List is also valid:

```json5
["Alice" "Bob" "Carol" "Dan"]
```

> In ASON, commas are optional and whitespace can be used as separators.

##### 5.2.1.2 Number of Elements

The number of elements in a List is variable, even a List with no elements (called an empty list) is valid:

```json5
[] // An empty list.
```

#### 5.2.2 Tuples

A Tuple is a collection of values with different data types, for example:

```json5
(11, "Alice", true)
```

As described above, commas in ASON are optional and whitespace can be used as separators, so the following Tuple are identical to the above Tuple:

```json5
(11 "Alice" true)
```

```json5
(
  11,
  "Alice",
  true,
)
```

```json5
(
  11
  "Alice"
  true
)
```

Tuples are different from Lists in that:

- The elements in a Tuple can be of different data types, while the elements in a List must be of the same data type.
- The amount of elements in a Tuple is fixed, while the amount of elements in a List is dynamic.
- Tuples are enclosed in parentheses (`(...)`), while Lists are enclosed in square brackets (`[...]`).

#### 5.2.3 Objects

An Object is a collection of key-value pairs, where the key is an identifier and the value can be any type.

```json5
{
    name: "ason",
    version: "1.0.1",
    edition: "2021"
}
```

The keys are _identifiers_ which are similar to strings but without quotation marks (`"`). An identifier must start with a letter (a-z, A-Z) or an underscore (`_`), followed by any combination of letters, digits (0-9), and underscores. For example, `name`, `version`, `_edition`, `foo_bar123` are all valid identifiers.

> Objects are also called Structs in some programming languages.

Key-value pairs in an Object can be separated by commas, whitespace, or line breaks, the following Objects are all identical:

```json5
// separated by commas
{ name: "ason", version: "1.0.1", edition: "2021" }

// separated by commas, and an additional tail comma
{ name: "ason", version: "1.0.1", edition: "2021",}

// separated by spaces
{ name: "ason" version: "1.0.1" edition: "2021" }

// separated by line breaks
{
    name: "ason"
    version: "1.0.1"
    edition: "2021"
}

// separated by both commas and line breaks
{
    name: "ason",
    version: "1.0.1",
    edition: "2021",
}
```

The values within an Object can be any type, including primitive values and compound values. In the real world, an Object usually contains other Objects and Lists, for example:

```json5
{
    name: "ason"
    version: "1.0.1"
    author: {
        name: "Alice"
        email: "alice@example.com"
    }
    dependencies: [
        "serde@1.0"
        "chrono@0.4"
    ]
}
```

#### 5.2.4 Named Lists

Named Lists are special Lists that each value is associated with a name, we call such elements "name-value pairs". The following is an example of a Named List which consists of string-string pairs:

```json5
[
    "serde": "1.0"
    "serde_bytes": "0.11"
    "chrono": "0.4.38"
]
```

The names are typically strings or numbers, other types are also allowed. For example, the following is a Named List that consists of integer-string pairs:

```json5
[
    0xff0000: "red"
    0x00ff00: "green"
    0x0000ff: "blue"
]
```

> Named Lists are also called Maps in some programming languages.

Named Lists are different from Objects in that:

- The keys in an Object are identifiers, while the names in a Named List can be of any type.
- The amount of key-value pairs in an Object is fixed, while the amount of name-value pairs in a Named List is dynamic.
- Objects are enclosed in curly braces (`{...}`), while Named Lists are enclosed in square brackets (`[...]`).

#### 5.2.5 Enumerations

An Enumeration is a custom data type that consists of a type name and a set of variants. Each variant can optionally carry a value. The following demonstrates an Enumeration type named `Option` with two variants: `None` and `Some`, where `None` does not carry a value, while `Some` carries a value of type integer:

```json
Option::None
Option::Some(11)
```

> Enumerations are also known as _Algebraic types_ or _Variants_ in some programming languages.

A Variant can carry a value of any type, such as List, Object or Tuple:

```json5
Option::Some([11, 13, 17])
```

```json5
Option::Some({
    id: 123
    name: "Alice"
})
```

```json5
Option::Some((1, "foo", true))
```

There are also Object-like and Tuple-like variants, these forms are more convenient when the variant carries multiple values, for example:

```json5
// Object-like variant
Shape::Rectangle{
    width: 307
    height: 311
}
```

and

```json5
// Tuple-like variant
Color::RGB(255, 127, 63)
```

#### 5.2.6 Type of Compound Values

Similar to primitive values, compound values also have their own types. When deserializing an ASON document into values of a programming language, the type of compound values must match the expected type, otherwise deserialization will fail.

> List requires all elements to be of the same type, if a List contains compound values, the type of the compound values must also be consistent.

##### 5.2.6.1 Type of Lists

The type of a List is determined by the type of its elements. For example, a List of integers has the type `[i32]`, a List of strings has the type `[String]`.

Since the amount of elements in a List is variable, the type of a List does not depend on the number of elements. For example, `[11, 13, 17]` and `[101, 103, 107, 109]` are both of type `[i32]`, even though the first List has 3 elements while the second List has 4 elements.

##### 5.2.6.2 Type of Tuples

The type of a Tuple is determined by the type and order of its elements. For example, the Tuple `(11, "Alice", true)` has the type `(i32, String, bool)`, while the Tuple `("Bob", 3.14)` has the type `(String, f64)`.

##### 5.2.6.3 Type of Objects

The type of an Object is determined by the keys and the corresponding value types. For example, the Object

```json5
{
    id: 123
    name: "Alice"
}
```

has the type `{ id: i32, name: String }`.

Note that some programming languages allow default values for missing fields in an Object, in such cases, the type of `{id: 123, name: "Alice"}` and `{id: 123}` can both be `{ id: i32, name: String }`, because the `name` field can be filled with a default value when it is missing.

But `{id: 123, user: "Bob"}` and `{id: 123, user: {name: "Bob", email: "bob@example.com"}}` are not of the same type, because the type of `user` field is different in the two Objects.

##### 5.2.6.4 Type of Named Lists

The type of a Named List is determined by the type of its name-value pairs. For example, the Named List

```json5
[
    "red": 0xff0000
    "green": 0x00ff00
    "blue": 0x0000ff
]
```

has the type `[String: i32]`.

##### 5.2.6.5 Type of Enumerations

The type of an Enumeration is determined by its type name. For example, the Enumeration `Option` has the type `Option`, and the Enumeration `Shape` has the type `Shape`.

### 5.3 Comments

Like JavaScript, C/C++ and Rust, ASON also supports two types of comments: line comments and block comments. Comments are for human readability and are completely ignored by the parser.

Line comments start with the `//` symbol and continue until the end of the line. For example:

```json5
// This is a line comment.
{
    id: 123     // This is another line comment.
    name: "Bob"
}
```

Block comments start with the `/*` symbol and end with the `*/` symbol. For example:

```json5
/* This is a block comment. */
{
    /*
     This is also a block comment.
    */
    id: 123 
    name: /* Definitely a block comment. */ "Bob"
}
```

Unlike JavaScript, C/C++ and Rust, ASON block comments support nesting. For example:

```json5
/* 
    This is the first level.
    /*
        This is the second level.
    */
    This is the first level again.
*/  
```

The nesting feature of block comments makes it more convenient for us to comment on a piece of code that **already has a block comment**.

### 5.4 Documents

The root of an ASON document can only be a single value, which can be either a primitive value or a compound value. A typical ASON document is usually an Object or a List, however, all types of values are allowed, such as a single number, a string, a Tuple, etc. The following two are valid ASON documents:

```json5
// The root is a Tuple.
(11, "Alice", true)
```

and

```json5
// The root is a string.
"Hello World!"
```

While the following two are invalid:

```json5
// There are 2 values at the root
(11, "Alice", true)
"Hello World!"
```

and

```json5
// There are 3 values at the root
11, "Alice", true
```

## 6 Mapping between Rust Data Types and ASON Types

ASON natively supports most Rust data types, including Tuples, Enums and Vectors. Because ASON is also strongly numeric typed, both serialization and deserialization can ensure data accuracy.

> ASON is perfectly compatible with Rust's data type system.

The following is a list of supported Rust data types:

- Signed and unsigned integers, from `i8`/`u8` to `i64`/`u64`
- Floating point numbers, including `f32` and `f64`
- Boolean
- Char
- String
- Array, such as `[i32; 4]`
- Vec
- Tuple
- Struct
- HashMap
- Enum

### 6.1 Structs

Rust structs correspond to ASON `Object`. The following is an example of a struct named "User":

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String
}
```

The following is an example ASON document for an instance of `User`:

```json5
{
    id: 123
    name: "John"
}
```

Real-world data is often complex, for example, a struct containing another struct to form a hierarchical relationship. The following code demonstrates struct `User` contains a child struct named `Address`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    address: Box<Address>
}

#[derive(Serialize, Deserialize)]
struct Address {
    city: String,
    street: String
}
```

An example ASON document for above struct `User` is:

```json5
{
    id: 123
    name: "John"
    address: {
        city: "Shenzhen"
        street: "Xin'an"
    }
}
```

### 6.2 Vecs

`Vec` corresponds to ASON `List`. The following code demonstrates adding a field named `orders` to the struct `User` to store order numbers:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    orders: Vec<i32>
}
```

The following is an example ASON document for an instance of `User`:

```json5
{
    id: 123
    name: "John"
    orders: [11, 13, 17, 19]
}
```

The elements in a vector can be also complex data, such as struct. The following code demonstrates adding a field named `addresses` to the struct `User`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    addresses: Vec<Address>
}

#[derive(Serialize, Deserialize)]
struct Address {
    city: String,
    street: String
}
```

An example ASON document for above struct `User` is:

```json5
{
    id: 123
    name: "John"
    addresses: [
        {
            city: "Guangzhou"
            street: "Tian'he"
        }
        {
            city: "Shenzhen"
            street: "Xin'an"
        }
    ]
}
```

### 6.3 HashMaps

Rust's HashMap corresponds to ASON's Map, e.g. the following is a HashMap with key type `String` and value type `i32`:

```rust
let mut m1 = HashMap::<String, i32>::new();
m1.insert("foo".to_owned(), 11);
m1.insert("bar".to_owned(), 22);
m1.insert("baz".to_owned(), 33);
```

The corresponding ASON document for instance `m1` is:

```json5
{
    "foo": 11
    "bar": 22
    "baz": 33
}
```

### 6.4 Tuples

Tuple in Rust can be considered as structs with omitted field names. Tuple just corresponds to ASON `Tuple`.

For example, the following code demonstrates a vector of tuples, where each tuple consists of an integer and a string:

```rust
let orders = vec![
    (11, String::from("ordered"),
    (13, String::from("shipped"),
    (17, String::from("delivered"),
    (19, String::from("cancelled")
]
```

The corresponding ASON document for instance `orders` is:

```json5
[
    (11, "ordered")
    (13, "shipped")
    (17, "delivered")
    (19, "cancelled")
]
```

In some programming languages, tuples and vectors are not clearly distinguished, but in Rust they are completely different data types. Vectors require that all elements have the same data type, while tuples require a fixed number and type of members. ASON's definition of `Tuple` is consistent with this convention.

### 6.5 Enums

Rust enum corresponds to ASON Enumeration. The following code defines an enum named `Color` with four variants: `Transparent`, `Grayscale`, `Rgb` and `Hsl`.

```rust
#[derive(Serialize, Deserialize)]
enum Color {
    Transparent,
    Grayscale(u8),
    Rgb(u8, u8, u8),
    Hsl{
        hue: i32,
        saturation: u8,
        lightness: u8
    }
}
```

Consider the following instance:

```rust
let e2 = vec![
    Color::Transparent,
    Color::Grayscale(127),
    Color::Rgb(255, 127, 63),
    Color::Hsl{
        hue: 300,
        saturation: 100,
        lightness: 50
    }
];
```

The corresponding ASON document for this instance is:

```json5
[
    Color::Transparent
    Color::Grayscale(127_u8)
    Color::Rgb(255_u8, 127_u8, 63_u8)
    Color::Hsl{
        hue: 300
        saturation: 100_u8
        lightness: 50_u8
    }
]
```

### 6.6 Other Data Types

Some Rust data types are not supported, includes:

- Unit (i.e. `()`)
- Unit struct, such as `struct Foo;`
- New-type struct, such as `struct Width(u32);`
- Tuple-like struct, such as `struct RGB(u8, u8, u8);`

According to the [serde framework's data model](https://serde.rs/data-model.html), it does not include the `DateTime` type, so ASON `DateTime` cannot be directly serialized or deserialized to Rust's `chrono::DateTime`. If you serialize a `chrono::DateTime` type value in Rust, you will get a regular string.

In addition, serde treats fixed-length arrays such as `[i32; 4]` as tuples rather than vectors, so the Rust array `[11, 13, 17, 19]` (type `[i32; 4]`) will be serialized as ASON Tuple `(11, 13, 17, 19)`.

### 6.7 Default Values

Rust serde framework allows you to specify default values for missing fields in a struct, this feature is also supported by ASON. For example:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    #[serde(default = "default_age")]
    age: u8
}
```

In this example, the `age` field has a default value of `u8`, so if the `age` field is missing in the ASON document, it will be filled with the default value during deserialization. For example, the following ASON document is valid and can be deserialized into an instance of `User`:

```json5
{
    id: 123
    name: "John"
}
```

## 7 ASON Specification

This section describes the ASON specification in detail, it is mainly for developers who want to implement ASON parsers and formatters in other programming languages. For general users, please refer to the previous sections.

[TODO]

## 8 Linking

- [Source code on GitHub](https://github.com/hemashushu/ason)
- [Crates.io](https://crates.io/crates/ason)

## 9 License

This project is licensed under the MPL 2.0 License with additional terms. See the files [LICENSE](./LICENSE) and [LICENSE.additional](./LICENSE.additional)
