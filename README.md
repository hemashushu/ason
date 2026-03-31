# ASON

_ASON_ is a data serialization format that evolved from JSON, featuring strong numeric typing and native support for enumeration types. With excellent readability and maintainability, ASON is well-suited for configuration files, data transfer, and data storage.

**Table of Content**

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->

- [1 ASON Example](#1-ason-example)
- [2 Comparison of Common Data Serialization Formats](#2-comparison-of-common-data-serialization-formats)
- [3 What improvements does ASON bring over JSON?](#3-what-improvements-does-ason-bring-over-json)
- [4 Library and APIs](#4-library-and-apis)
  - [4.1 Deserialization and Serialization](#41-deserialization-and-serialization)
  - [4.2 Streaming Deserialization and Serialization](#42-streaming-deserialization-and-serialization)
  - [4.3 Parser and Writer](#43-parser-and-writer)
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
  - [7.1 Document Structure](#71-document-structure)
  - [7.2 Lexical Elements](#72-lexical-elements)
    - [7.2.1 Encoding](#721-encoding)
    - [7.2.2 Whitespace](#722-whitespace)
    - [7.2.3 Commas](#723-commas)
    - [7.2.4 Comments](#724-comments)
    - [7.2.5 Punctuation](#725-punctuation)
  - [7.3 Primitive Values](#73-primitive-values)
    - [7.3.1 Numbers](#731-numbers)
    - [7.3.2 Booleans](#732-booleans)
    - [7.3.3 Characters](#733-characters)
    - [7.3.4 Strings](#734-strings)
    - [7.3.5 DateTime](#735-datetime)
    - [7.3.6 Hexadecimal Byte Data](#736-hexadecimal-byte-data)
  - [7.4 Compound Values](#74-compound-values)
    - [7.4.1 Lists](#741-lists)
    - [7.4.2 Named Lists](#742-named-lists)
    - [7.4.3 Tuples](#743-tuples)
    - [7.4.4 Objects](#744-objects)
    - [7.4.5 Enumerations](#745-enumerations)
  - [7.5 Grammar Summary](#75-grammar-summary)
  - [7.6 Token Termination](#76-token-termination)
  - [7.7 Processing Pipeline](#77-processing-pipeline)
  - [7.8 Error Handling](#78-error-handling)
  - [7.9 File Extension and MIME Type](#79-file-extension-and-mime-type)
- [8 Linking](#8-linking)
- [9 License](#9-license)

<!-- /code_chunk_output -->

## 1 ASON Example

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

## 2 Comparison of Common Data Serialization Formats

There are many solid data serialization formats available today, such as JSON, YAML, TOML and XML. They are all designed to be readble and writable by both humans and machines.

The differences between these formats are minor for small datasets, but become more pronounced as datasets grow or structures become more complex. For example, YAML's indentation-based syntax can cause errors in editing large documents, and TOML's limited support for complex structures can make representing hierarchical data cumbersome. JSON, by contrast, is designed to stay simple, consistent, and expressive at any scale.

For developers, JSON offers additional advantages:

- You don't have to learn an entirely new syntax: JSON closely resembles JavaScript object literals.
- Implementing a parser is straightforward, which helps ensure longevity and adaptability across evolving software ecosystems.

## 3 What improvements does ASON bring over JSON?

JSON has a simple syntax and has been around for decades, but it struggles to meet diverse modern needs. Many JSON variants have emerged to address its limitations—such as JSONC (which adds comments) and JSON5 (which allows trailing commas and unquoted object keys). However, these variants still cannot represent data accurately due to JSON has a limited type system and lacks fine-grained numeric and domain-specific data types. ASON takes a significant step forward based on JSON with the following improvements:

- **Explicit Numeric Types:** ASON numbers can be explicitly typed (e.g., `u8`, `i32`, `f32`, `f64`) ensuring more precise and rigirous data representation. Additionally, integers can be represented in hexadecimal, octal, and binary formats.
- **New Data Types:** New data types such as `Char`, `DateTime`, and `HexadecimalByteData` to better represent common data types.
- **More string formats:** "Multi-line strings", "Concatenate strings", "Raw strings", and "Auto-trimmed strings" are added to enhance string representation.
- **Separate List and Tuple:** ASON distinguishes between `List` (homogeneous elements) and `Tuple` (heterogeneous elements), enhancing data structure clarity.
- **Separate Named-List and Object:** ASON introduces `Named-List` (also called `Map`) alongside `Object` (also called `Struct`), enhancing data structure clarity in further.
- **Native Enumerations Support:** ASON natively supports enumerations types (also known as _Algebraic types_ or _Variants_). This enables seamless serialization of complex data structures from high-level programming languages.
- **Eliminating the Null Value:** ASON uses the `Option` enumeration to represent optional values, eliminating the error-prone `null` value.
- **Simple and Consistent:** ASON supports comments, unquoted object field names, trailing commas, and whitespace-separated elements (in addition to commas). These features enhance writing fluency.

In addition to the text format, ASON provides a binary format called [ASONB (ASON Binary)](https://github.com/hemashushu/asonb),  ASONB supports both incremental updates and streaming and random access, making it suitable for a wide range of applications, from simple data storage to complex data processing pipelines.

While ASON is designed to resemble JSON, making it easy for JSON users to learn and adopt, it is not compatible with JSON, but conversion between ASON and JSON is straightforward, and implementing an ASON parser is also simple.

## 4 Library and APIs

The Rust [ason](https://github.com/hemashushu/ason) library provides three set APIs:

1. [Serde](https://github.com/serde-rs/serde) based APIs for deserialization and serialization.
2. AST (Abstract Syntax Tree) based APIs for parsing and writing ASON documents.

In general, it is recommended to use the serde API since it is simple enough to meet most needs.

### 4.1 Deserialization and Serialization

Consider the following ASON document:

```json5
{
    name: "foo"
    version: "0.1.0"
    dependencies: [
        "random"
        "regex"
    ]
}
```

This document consists of an object and a list. The object has `name`, `version` and `dependencies` fields, and the list has strings as elements. We can create a Rust struct corresponding to this document:

```rust
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
}
```

The struct needs to be annotated with `Serialize` and `Deserialize` traits (which are provided by the _serde_ serialization framework) to enable serialization and deserialization.

The following code demonstrates how to use the function `ason::de::de_from_str` to deserialize the ASON document string into a `Package` struct instance:

```rust
// The above ASON document
let text = "...";

// Deserialize the ASON document string into a `Package` struct.
let package: Package = ason::de::de_from_str(text).unwrap();

// Verify the deserialized `Package` struct.
assert_eq!(
    package,
    Package {
        name: String::from("foo"),
        version: String::from("0.1.0"),
        dependencies: vec![
            String::from("random"),
            String::from("regex")
        ],
    }
);
```

You can serialize the `Package` struct instance back into an ASON document string using the `ason::ser::ser_to_string` function:

```rust
// Serialize the `Package` struct back into an ASON document string.
let serialized_text = ason::ser::ser_to_string(&package).unwrap();
assert_eq!(serialized_text, text);
```

### 4.2 Streaming Deserialization and Serialization

The `ason::de` module also provides streaming deserialization APIs, which allow you to deserialize ASON documents incrementally without loading the entire document into memory. This is particularly useful for large documents or the documents are transmitted over a network and pipe.

But only documents only contain a `List` can be deserialized using the streaming deserialization APIs, and the deserialized elements are returned one by one as an iterator. For example:

```rust
let s = r#"[11 13]"#;
let data = s.as_bytes();
let mut char_iter = UTF8CharIterator::new(data);
let mut de = list_from_char_iterator(&mut char_iter).unwrap();

assert_eq!(11, de.next().unwrap().unwrap());
assert_eq!(13, de.next().unwrap().unwrap());
assert!(de.next().is_none());

```

### 4.3 Parser and Writer

You can parse the ASON document into AST object using the `ason::parser::parse_from_str` function:

```rust
// The above ASON document
let text = "...";

// Convert the ASON document string to an AST node.
let node = ason::parser::parse_from_str(text).unwrap();

// Verify the AST node structure.
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
                AsonNode::String(String::from("random")),
                AsonNode::String(String::from("regex"))
            ]))
        }
    ])
);
```

You can also turn the AST object into a string using the `ason::writer::write_to_string` function:

```rust
// Convert the AST node back to an ASON document string.
let document = ason::writer::write_to_string(&node);
assert_eq!(document, text);
```

Since AST object lacks some information such as comments, whitespace, the original string format (e.g., multi-line string, raw string, etc.), and the original numeric types (e.g., hexadecimal, octal, binary), so the output text may not be exactly the same as the input text, do not use the writer for formatting ASON documents.

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

Underscores may also appear between the number and its type suffix for readability: `933_199_u32`, `6.626e-34_f32`

> If no explicit type is specified, integers default to `i32` and floating-point numbers default to `f64`.

#### 5.1.3 Hexadecimal, Octal and Binary Integers

Beyond decimal notation, ASON supports three additional integer formats:

- Hexadecimal: prefix with `0x` (e.g., `0xFF`)
- Octal: prefix with `0o` (e.g., `0o755`)
- Binary: prefix with `0b` (e.g., `0b1010`)

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
    concatenated_string: "My \
        very educated \
        mother \
        just served \
        us \
        nine pizzas"
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

`"""\n...\n optional_whitespace """`

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

The keys are identifiers which are similar to strings but without quotation marks (`"`). An identifier must start with a letter (a-z, A-Z) or an underscore (`_`), followed by any combination of letters, digits (0-9), and underscores. For example, `name`, `version`, `_edition`, `foo_bar123` are all valid identifiers.

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

> Objects are also called Structs in some programming languages.

#### 5.2.4 Named Lists

Named Lists are special Lists that each value is associated with a name, we call such elements name-value pairs. The following is an example of a Named List which consists of string-string pairs:

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
    5: "Perfect"
    4: "Good"
    3: "Fair"
    2: "Poor"
    1: "Terrible"
]
```

Named Lists are different from Objects in that:

- The keys in an Object are identifiers, while the names in a Named List can be of any type.
- The amount of key-value pairs in an Object is fixed, while the amount of name-value pairs in a Named List is dynamic.
- Objects are enclosed in curly braces (`{...}`), while Named Lists are enclosed in square brackets (`[...]`).

> Named Lists are also called Maps in some programming languages.

#### 5.2.5 Enumerations

An Enumeration is a custom data type that consists of a type name and a set of variants. Each variant can optionally carry a value. The following demonstrates an Enumeration type named `Option` with two variants: `None` and `Some`, where `None` does not carry a value, while `Some` carries a value of type integer:

```json
Option::None
Option::Some(11)
```

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

> Enumerations are also known as _Algebraic types_ or _Variants_ in some programming languages.

#### 5.2.6 Type of Compound Values

Similar to primitive values, compound values also have their own types. When deserializing an ASON document into values of a programming language, the type of compound values must match the expected type, otherwise deserialization will fail.

> List requires all elements to be of the same type, if a List contains compound values, the type of the compound values must also be consistent.

##### 5.2.6.1 Type of Lists

The type of a List is determined by the type of its elements. For example, a List of integers has the type `[i32]`, a List of strings has the type `[String]`.

Since the amount of elements in a List is variable, the type of a List does not depend on the number of elements. For example, `[11, 13, 17]` and `[101, 103, 107, 109]` are both of type `[i32]`, even though the first List has 3 elements while the second List has 4 elements.

The type of empty List is determined by the context in which it is used.

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

The type of a Named List is determined by the type of its name-value pairs. For example, the Named List:

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

The nesting feature of block comments makes it more convenient for us to comment on a piece of code that already has a block comment.

### 5.4 Documents

The root of an ASON document can only be a single value, which can be either a primitive value or a compound value. A typical ASON document is usually an Object or a List, however, all types of values are allowed, such as a single number, a string, a Tuple, etc. The following two are valid ASON documents:

```json5
// The root value is a Tuple.
(11, "Alice", true)
```

and

```json5
// The root value is a string.
"Hello World!"
```

While the following two are invalid:

```json5
// There are two values at the root level
(11, "Alice", true)
"Hello World!"
```

and

```json5
// There are three values at the root level
11, "Alice", true
```

## 6 Mapping between Rust Data Types and ASON Types

ASON natively supports most Rust data types, including Tuples, Enums and Vectors. Because ASON is also strongly numeric typed, both serialization and deserialization can ensure data accuracy.

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

> ASON is perfectly compatible with Rust's data type system.

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
    (11, String::from("ordered")),
    (13, String::from("shipped")),
    (17, String::from("delivered")),
    (19, String::from("cancelled"))
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

This section describes the ASON specification in detail. It is mainly intended for developers who want to implement ASON parsers and formatters in other programming languages. For general users, please refer to the previous sections.

### 7.1 Document Structure

An ASON document consists of exactly one value at the root level. The root value can be any valid ASON value: a primitive value (Number, String, Boolean, etc.) or a compound value (Object, List, Tuple, etc.).

A document containing zero value or more than one value at the root level is invalid.

Valid documents:

```json5
// A single object as root
{id: 123, name: "Alice"}
```

```json5
// A single string as root
"Hello World!"
```

```json5
// A single number as root
42
```

Invalid documents:

```json5
// Two values at root level (invalid)
(11, "Alice", true)
"Hello World!"
```

```json5
// Three values at root level (invalid)
11, "Alice", true
```

### 7.2 Lexical Elements

#### 7.2.1 Encoding

ASON documents are encoded in UTF-8.

#### 7.2.2 Whitespace

The following characters are treated as whitespace and are used to separate tokens. They are ignored by the parser (except within strings and characters):

| Character       | Code Point |
|-----------------|------------|
| Space           | U+0020     |
| Tab             | U+0009     |
| Carriage Return | U+000D     |
| Line Feed       | U+000A     |

The "carriage return - line feed" pair (`\r\n`, Windows-style) is treated as a single line break.

#### 7.2.3 Commas

Commas (`,`) serve as optional separators between elements of Lists, Tuples, Objects, and Named Lists. They carry no semantic meaning and are treated as whitespace by the parser. Trailing commas (after the last element) are permitted.

The following are all equivalent:

```json5
[1,2,3]
[1 2 3]
[1,2,3,]
```

#### 7.2.4 Comments

ASON supports two types of comments:

**Line comments** start with `//` and continue to the end of the line:

```json5
// This is a line comment.
123 // This is also a line comment.
```

**Block comments** start with `/*` and end with `*/`:

```json5
/* This is a block comment. */
```

Block comments support nesting. The start and end markers must appear in matched pairs:

```json5
/* outer /* inner */ outer again */
```

Comments are completely ignored by the parser and carry no semantic meaning.

Inside a line comment, the characters `/*` and `*/` have no special meaning. Similarly, inside a block comment, the characters `//` have no special meaning (but `/*` and `*/` do, due to nesting).

#### 7.2.5 Punctuation

The following punctuation characters are significant tokens:

| Token | Meaning                                            |
|-------|----------------------------------------------------|
| `{`   | Opening brace (Objects)                            |
| `}`   | Closing brace (Objects)                            |
| `[`   | Opening bracket (Lists and Named Lists)            |
| `]`   | Closing bracket (Lists and Named Lists)            |
| `:`   | Key-value separator (Objects and Named Lists)      |
| `(`   | Opening parenthesis (Tuples)                       |
| `)`   | Closing parenthesis (Tuples)                       |
| `+`   | Positive sign (Numbers)                            |
| `-`   | Negative sign (Numbers)                            |
| `::`  | Enumeration separator (Type name and variant name) |

### 7.3 Primitive Values

#### 7.3.1 Numbers

ASON supports integer and floating-point numbers with explicit type annotations. Each number has a specific data type.

##### 7.3.1.1 Number Types

The supported number types are:

| Type  | Description                      | Width   | Range                    |
|-------|----------------------------------|---------|--------------------------|
| `i8`  | Signed 8-bit integer             | 8 bits  | -128 to 127              |
| `u8`  | Unsigned 8-bit integer           | 8 bits  | 0 to 255                 |
| `i16` | Signed 16-bit integer            | 16 bits | -32,768 to 32,767        |
| `u16` | Unsigned 16-bit integer          | 16 bits | 0 to 65,535              |
| `i32` | Signed 32-bit integer            | 32 bits | -2,147,483,648 to 2,147,483,647 |
| `u32` | Unsigned 32-bit integer          | 32 bits | 0 to 4,294,967,295       |
| `i64` | Signed 64-bit integer            | 64 bits | -2^63 to 2^63 - 1        |
| `u64` | Unsigned 64-bit integer          | 64 bits | 0 to 2^64 - 1            |
| `f32` | 32-bit floating-point (IEEE 754) | 32 bits | ±3.4028235e+38           |
| `f64` | 64-bit floating-point (IEEE 754) | 64 bits | ±1.7976931348623157e+308 |

Default types:

- Integers without an explicit type suffix default to `i32`.
- Floating-point numbers without an explicit type suffix default to `f64`.

##### 7.3.1.2 Decimal Integers

Decimal integers consist of one or more decimal digits (`0`-`9`).

```text
decimal_integer = "0" | ( non_zero_digit { digit } )
non_zero_digit  = "1" | "2" | ... | "9"
digit           = "0" | "1" | ... | "9"
```

Examples: `0`, `123`, `999_999`

Leading zeros are not permitted for multi-digit decimal integers. `0` alone is valid, but `0123` is invalid.

##### 7.3.1.3 Hexadecimal Integers

Hexadecimal integers are prefixed with `0x` or `0X`, followed by one or more hexadecimal digits (`0`-`9`, `a`-`f`, `A`-`F`).

Examples: `0xFF`, `0x1A2B`, `0x00`

```text
hex_integer = "0" ("x" | "X") hex_digit { hex_digit }
hex_digit   = "0"..."9" | "a"..."f" | "A"..."F"
```

The prefix is case-insensitive (both `0x` and `0X` are valid). An empty hexadecimal number (e.g., `0x`) is invalid.

##### 7.3.1.4 Octal Integers

Octal integers are prefixed with `0o` or `0O`, followed by one or more octal digits (`0`-`7`).

```text
octal_integer = "0" ("o" | "O") octal_digit { octal_digit }
octal_digit   = "0"..."7"
```

Examples: `0o755`, `0o644`

An empty octal number (e.g., `0o`) is invalid. Octal floating-point numbers are not supported.

##### 7.3.1.5 Binary Integers

Binary integers are prefixed with `0b` or `0B`, followed by one or more binary digits (`0` or `1`).

```text
binary_integer = "0" ("b" | "B") binary_digit { binary_digit }
binary_digit   = "0" | "1"
```

Examples: `0b1010`, `0b1100_0011`

An empty binary number (e.g., `0b`) is invalid. Binary floating-point numbers are not supported.

##### 7.3.1.6 Decimal Floating-Point Numbers

A decimal floating-point number contains either a decimal point (`.`) or an exponent part (introduced by `e` or `E`), or both.

```text
decimal_float = digits "." digits [ exponent ]
              | digits exponent
exponent      = ("e" | "E") [ "+" | "-" ] digits
digits        = digit { digit }
```

Examples: `3.14`, `2.998e8`, `6.626e-34`, `1.0e+3`

Rules:

- A number must not start or end with a decimal point. For example, `.123` and `123.` are both invalid.
- A number must not end with `e` or `E`. For example, `e123` and `123e` are invalid.
- Multiple decimal points or exponent markers are not allowed.
- The default type is `f64`.

##### 7.3.1.7 Hexadecimal Floating-Point Numbers

Hexadecimal floating-point numbers use the [P notation](https://en.wikipedia.org/wiki/Hexadecimal#Exponential_notation) format (also known as hexadecimal exponential notation). The format is:

```text
hex_float = "0x" hex_digits "." hex_digits "p" [ "+" | "-" ] digits
```

Where the significand is hexadecimal and the exponent (after `p`) is a decimal power of 2.

Examples: `0x1.921fb6p1` (≈ π as f32), `0x1.5bf0a8b145769p+1` (≈ e as f64), `0x1.4p3` (= 10.0)

Rules:

- The exponent part (`p` or `P` followed by a decimal integer) is mandatory. A number like `0x1.23` without the `p` part is invalid.
- A number must not end with `.` or `p`. For example, `0x1.` and `0x1p` are both invalid.
- The default type is `f64`.
- Only `f32` and `f64` type suffixes are valid for hexadecimal floating-point numbers. Integer type suffixes (e.g., `i32`, `u32`) are not permitted.

> Note: Appending a type suffix directly to a hexadecimal integer creates ambiguity because `f` is a valid hexadecimal digit. For example, `0x21_f32` is parsed as the hexadecimal integer `0x21f32`, not a typed floating-point number. Use P notation for hexadecimal floating-point values.

##### 7.3.1.8 Digit Separators

Underscores (`_`) may be inserted between any two digits in a numeric literal to improve readability. They are ignored by the parser.

Examples: `123_456_789`, `0xFF_FF`, `0b1010_0101`, `6.626_070_e-34`

##### 7.3.1.9 Type Suffixes

An explicit type suffix may be appended to a number to specify its type. The suffix consists of one of the type names listed in section 7.3.1.1 (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`).

One or more underscores may appear between the number and its type suffix: `255_u8`, `3.14_f32`, `65u8`.

Rules for type suffixes with different number formats:

- Decimal integers and floating-point numbers: All type suffixes are valid.
- Hexadecimal integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. The `f32` and `f64` suffixes cannot be used because `f` is a valid hexadecimal digit.
- Hexadecimal floating-point numbers (P notation): Only `f32` and `f64` suffixes are valid.
- Octal and binary integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. Floating-point types are not supported for these formats.

If a number's value exceeds the range of the specified type, it is an error.

##### 7.3.1.10 Signed Numbers

Numbers can be preceded by a `+` (plus) or `-` (minus) sign.

Rules:

- The `+` sign is permitted in front of all signed types and floating-point types. It is a no-op (the number keeps its positive value). A `+` sign in front of a `NaN` is invalid.
- The `-` sign negates the number. It is permitted for signed integer types (`i8`, `i16`, `i32`, `i64`) and floating-point types (`f32`, `f64`). Applying `-` to unsigned types (`u8`, `u16`, `u32`, `u64`) is an error. A `-` sign in front of `NaN` is also invalid.
- Overflow after applying the sign is checked. For example, `128_i8` is invalid because `128` exceeds `i8::MAX` (127), but `-128_i8` is valid because the i8 range includes -128.

##### 7.3.1.11 Special Floating-Point Values

ASON supports the special floating-point values `NaN` (Not a Number) and `Inf` (Infinity):

| Literal    | Value                 |
|------------|-----------------------|
| `NaN`      | f64 NaN               |
| `NaN_f32`  | f32 NaN               |
| `NaN_f64`  | f64 NaN               |
| `Inf`      | f64 positive infinity |
| `Inf_f32`  | f32 positive infinity |
| `Inf_f64`  | f64 positive infinity |
| `+Inf`     | f64 positive infinity |
| `-Inf`     | f64 negative infinity |
| `+Inf_f32` | f32 positive infinity |
| `-Inf_f32` | f32 negative infinity |

Rules:

- `NaN` does not allow a leading sign. Both `+NaN` and `-NaN` are invalid.
- `Inf` allows a leading `+` or `-` sign.
- Only `f32` and `f64` suffixes are valid for `NaN` and `Inf`. Other suffixes (e.g., `Inf_i32`, `NaN_i32`) are treated as regular identifiers.
- The default type is `f64`.

#### 7.3.2 Booleans

Boolean values are represented by the keywords `true` and `false`.

```text
boolean = "true" | "false"
```

#### 7.3.3 Characters

A character literal consists of a single character enclosed in single quotes (`'`).

```text
char = "'" ( character | escape_sequence ) "'"
```

The character can be any valid Unicode scalar value (code points U+0000 to U+D7FF and U+E000 to U+10FFFF).

An empty character (`''`) is invalid.

A character containing more than one character (e.g., `'ab'`) is also invalid. Some emojis may be composed of multiple Unicode characters, although they appear to be a single character when displayed. For example, the emoji 🤦‍♂️ is actually composed of 4 characters U+1F926, U+200D, U+2642, U+FE0F. If this emoji is enclosed in single quotes, it is an invalid character.

##### 7.3.3.1 Escape Sequences

The following escape sequences are supported in both character and string literals:

| Escape Sequence | Character                    |
|-----------------|------------------------------|
| `\\`            | Backslash (`\`)              |
| `\'`            | Single quote (`'`)           |
| `\"`            | Double quote (`"`)           |
| `\t`            | Horizontal tab (U+0009)      |
| `\n`            | Line feed / newline (U+000A) |
| `\r`            | Carriage return (U+000D)     |
| `\0`            | Null character (U+0000)      |
| `\u{HHHHHH}`    | Unicode code point           |

Unicode escape sequences use the format `\u{H...}`, where `H...` is 1 to 6 hexadecimal digits representing a Unicode code point. The braces are mandatory. Examples: `\u{2d}` (character hyphen `-`), `\u{6587}` (CJK character `文`).

Invalid Unicode code points (values greater than U+10FFFF or surrogate code points U+D800 to U+DFFF) are errors.

The following escape sequences from other languages are not supported:

| Escape Sequence | Description                      |
|-----------------|----------------------------------|
| `\a`            | Alert (U+0007)                   |
| `\b`            | Backspace (U+0008)               |
| `\v`            | Vertical tab (U+000B)            |
| `\f`            | Form feed (U+000C)               |
| `\e`            | Escape character (U+001B)        |
| `\?`            | Question mark (U+003F)           |
| `\ooo`          | Octal character escapes          |
| `\xHH`          | Hexadecimal character escapes    |
| `\uHHHH`        | Unicode code point below 0x10000 |
| `\Uhhhhhhhh`    | Unicode code point               |

#### 7.3.4 Strings

A string literal is a sequence of characters enclosed in double quotes (`"`).

```text
string = '"' { string_char } '"'
string_char = any_character_except_backslash_and_double_quote
            | escape_sequence
```

The same escape sequences as character literals are supported (see section 7.3.3.1). Double quotes within the string must be escaped as `\"` and backslashes as `\\`.

An empty string (`""`) is valid.

##### 7.3.4.1 Multi-Line Strings

Strings can span multiple lines. All characters between the opening and closing double quotes (including line breaks, tabs, and spaces) are part of the string content.

```json5
"Line one
    Line two
    Line three"
```

This is equivalent to `"Line one\n    Line two\n    Line three"`. Note that all whitespace characters are preserved as-is.

##### 7.3.4.2 Concatenated Strings

A backslash (`\`) at the end of a line (immediately before a line break) enables string concatenation. The line break and all leading whitespace on the next line are removed, and the text continues without a break.

```json5
"The quick brown fox \
    jumps over \
    the lazy dog"
```

This is equivalent to `"The quick brown fox jumps over the lazy dog"`.

The backslash must be immediately followed by `\n` or `\r\n`. The leading whitespace (spaces and tabs) on the continuation line is stripped.

##### 7.3.4.3 Raw Strings

Raw strings use the prefix `r` before the opening double quote: `r"..."`. Within a raw string, escape sequences are disabled (backslashes and all other characters are treated literally).

```json5
r"^\d*(\.\d+)?$"
```

This is equivalent to `"^\\d*(\\.\\d+)?$"`.

A variant `r#"..."#` uses hash delimiters, allowing the string to contain unescaped double quotes:

```json5
r#"<a href="https://hemashushu.github.io/" title="Home">Home Page</a>"#
```

This is equivalent to `"<a href=\"https://hemashushu.github.io/\" title=\"Home\">Home Page</a>"`.

##### 7.3.4.4 Auto-Trimmed Strings

Auto-trimmed strings use triple double quotes (`"""`) and automatically remove common leading whitespace from each line. The syntax is:

```text
auto_trimmed_string = '"""' newline { content_line } newline { optional_whitespace } '"""'
```

Rules:

- The opening `"""` must be immediately followed by a line break (`\n` or `\r\n`).
- The closing `"""` must start on a new line. Leading whitespace before the closing `"""` is allowed but is not part of the content.
- The parser determines the minimum number of leading whitespace characters across all non-empty content lines and removes that many leading characters from each line.
- Empty lines (lines containing only a line break) and blank lines (lines containing only whitespace) are not considered when calculating the minimum indentation.
- The trailing line break before the closing `"""` is not part of the string content.

Example:

```json5
"""
    Hello
      World
    Goodbye
"""
```

This produces the string `"Hello\n  World\nGoodbye"`. Each line had at least 4 leading spaces, so 4 spaces are removed from each line.

Another example:

```json5
["""
    Hello
""", """
    Earth
      &
    Mars
"""]
```

The two strings are equivalent to `"Hello"` and `"Earth\n  &\nMars"`.

Note: Within auto-trimmed strings, escape sequences are **not** processed. All characters (including backslashes) are preserved as-is (similar to raw strings), except for the trimming of common leading whitespace.

#### 7.3.5 DateTime

DateTime values use the prefix `d` before a quoted date-time string:

```text
datetime = 'd"' date_time_string '"'
```

The supported formats are:

| Format                      | Example                        | Description                                |
|-----------------------------|--------------------------------|--------------------------------------------|
| `YYYY-MM-DD`                | `d"2024-03-16"`                | Date only (time defaults to 00:00:00, timezone defaults to UTC) |
| `YYYY-MM-DD HH:mm:ss`       | `d"2024-03-16 16:30:50"`       | Date and time (timezone defaults to UTC)   |
| `YYYY-MM-DDtHH:mm:ss`       | `d"2024-03-16t16:30:50"`       | Date and time with lowercase `t` separator |
| `YYYY-MM-DDTHH:mm:ssZ`      | `d"2024-03-16T16:30:50Z"`      | Date and time in UTC                       |
| `YYYY-MM-DDTHH:mm:ss±HH:MM` | `d"2024-03-16T16:30:50+08:00"` | Date and time with timezone offset         |

The date-time string is parsed according to [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339). The `T` or `t` separator between date and time is optional (a space is also accepted), and the [Corrdinated Universal Time (UTC)](https://en.wikipedia.org/wiki/Coordinated_Universal_Time) time zone can be appended to the date-time string. `Z` (uppercase) or `z` (lowercase) indicates that the timezone is in UTC (`+00:00`). If no timezone is specified, it is assumed to be UTC.

Characters allowed within the date-time string: digits (`0`-`9`), `-`, `:`, space (` `), `t`, `T`, `z`, `Z`, `+`.

#### 7.3.6 Hexadecimal Byte Data

Hexadecimal byte data is a sequence of bytes represented as two-digit hexadecimal values, prefixed with `h`:

```text
hex_byte_data = 'h"' { hex_byte } '"'
hex_byte      = hex_digit hex_digit
```

Rules:

- Each byte is exactly two hexadecimal digits. Single-digit values must be zero-padded (e.g., `a` should be written as `0a`).
- Bytes must be separated by one or more whitespace characters (spaces, tabs, or line breaks).
- Hexadecimal digits are case-insensitive (`a`-`f` and `A`-`F` are both valid).
- Leading and trailing whitespace within the quotes is permitted.
- An empty byte data literal (`h""`) is valid and represents zero bytes.

Examples:

```json5
h"48 65 6C 6C 6F"
h"11 13 17 19"
h""
```

Multi-line byte data:

```json5
h"48 65 6C 6C  6F 2C 20 57
  6F 72 6C 64  21 20 49 20
  61 6D 20 41  53 4F 4E 2E"
```

### 7.4 Compound Values

#### 7.4.1 Lists

A List is an ordered sequence of zero or more values enclosed in square brackets (`[` and `]`).

```text
list = "[" { value [","] } "]"
```

All elements in a List must be of the same type. For example, a list of integers and a list of strings are valid, but a list mixing integers and strings is invalid.

An empty List (`[]`) is valid.

Elements can be separated by commas, whitespace, or both. Trailing commas are permitted.

Examples:

```json5
[11, 13, 17, 19]
["Alice" "Bob" "Carol"]
[
    {name: "foo"}
    {name: "bar"}
]
[]
```

#### 7.4.2 Named Lists

A Named List (also called a Map) is an ordered sequence of name-value pairs enclosed in square brackets. It is distinguished from a regular list by the presence of a colon (`:`) after the first element.

```text
named_list = "[" { name_value_pair [","] } "]"
name_value_pair = value ":" value
```

The names (left side of `:`) can be of any type (strings, numbers, etc.), and all names must be of the same type. Similarly, all values must be of the same type.

Examples:

```json5
[
    "foo": 11,
    "bar": 22,
    "baz": 33
]
[
    0xff0000: "red",
    0x00ff00: "green",
    0x0000ff: "blue"
]
```

An empty named list is represented as an empty list `[]`.

#### 7.4.3 Tuples

A tuple is an ordered sequence of one or more values enclosed in parentheses (`(` and `)`).

```text
tuple = "(" value { [","] value } ")"
```

Unlike lists, tuples can contain elements of different types. The number and types of elements define the tuple's type.

An empty tuple (`()`) is **not** valid.

Elements can be separated by commas, whitespace, or both.

Examples:

```json5
(11, "Alice", true)
(3.14, 42_u8)
```

#### 7.4.4 Objects

An object is an ordered sequence of zero or more key-value pairs enclosed in curly braces (`{` and `}`).

```text
object = "{" { key_value_pair [","] } "}"
key_value_pair = identifier ":" value
```

Keys are identifiers (unquoted). They must **not** be enclosed in double quotes (unlike JSON). If a quoted string is used as a key, it is a parse error.

An identifier follows these rules:

- Must start with a letter (`a`-`z`, `A`-`Z`), an underscore (`_`), or a Unicode character in the range U+00A0 to U+D7FF or U+E000 to U+10FFFF (which includes CJK characters, Greek letters, emoji, etc.).
- May continue with letters, digits (`0`-`9`), underscores, or Unicode characters in the same ranges.
- Must not contain `::`. The sequence `::` within an identifier turns it into an enumeration token (see section 7.4.5).

An empty object in some programming languages may be invalid, but in ASON, an empty object (`{}`) is valid and represents an object with default values for all fields.

Key-value pairs can be separated by commas, whitespace, or both. Trailing commas are permitted.

Examples:

```json5
{id: 123, name: "Alice"}
{
    name: "ason"
    version: "1.0.1"
    edition: "2021"
}
{}
```

#### 7.4.5 Enumerations

An enumeration is a value consisting of a type name and a variant name, separated by `::`. It can optionally carry associated data.

```text
enumeration = type_name "::" variant_name [ variant_body ]
variant_body = "(" value { [","] value } ")"   // tuple-like or single value
             | "{" { key_value_pair [","] } "}" // object-like
```

Both the type name and variant name follow the same rules as identifiers.

There are four forms of enumeration values:

- Variant without associated data:

```json5
Option::None
Color::Red
```

- Single value variant, one value in parentheses:

```json5
Option::Some(123)
Option::Some("hello")
```

- Tuple-like variant, multiple values in parentheses:

```json5
Color::RGB(255, 127, 63)
```

- Object-like variant, key-value pairs in curly braces:

```json5
Shape::Rect{width: 200, height: 100}
```

An empty parentheses variant (`Option::Some()`) is **not** valid. If a variant carries parentheses, at least one value must be present.

> Note: The distinction between a single value variant (form 2) and a tuple-like variant (form 3) depends on the number of values within the parentheses. If there is exactly one value, it is a single value variant. If there are two or more values, it is a tuple-like variant.

### 7.5 Grammar Summary

The following is a summary of the ASON grammar in EBNF-like notation:

```ebnf
document            = value ;

value               = number
                    | boolean
                    | char
                    | string
                    | datetime
                    | hex_byte_data
                    | enumeration
                    | list
                    | named_list
                    | tuple
                    | object ;

(* Numbers *)
number              = [ "+" | "-" ] unsigned_number ;
unsigned_number     = decimal_number
                    | hex_number
                    | octal_number
                    | binary_number
                    | special_float ;
decimal_number      = digits [ "." digits ] [ exponent ] [ type_suffix ] ;
hex_number          = "0" ("x"|"X") hex_digits
                      [ "." hex_digits "p" ["+"|"-"] digits ] [ type_suffix ] ;
octal_number        = "0" ("o"|"O") octal_digits [ type_suffix ] ;
binary_number       = "0" ("b"|"B") binary_digits [ type_suffix ] ;
special_float       = "NaN" [ "_f32" | "_f64" ]
                    | "Inf" [ "_f32" | "_f64" ] ;
exponent            = ("e"|"E") ["+"|"-"] digits ;
type_suffix         = ["_"] ("i8"|"u8"|"i16"|"u16"|"i32"|"u32"|"i64"|"u64"|"f32"|"f64") ;
digits              = digit { ["_"] digit } ;
hex_digits          = hex_digit { ["_"] hex_digit } ;
octal_digits        = octal_digit { ["_"] octal_digit } ;
binary_digits       = binary_digit { ["_"] binary_digit } ;

(* Boolean *)
boolean             = "true" | "false" ;

(* Character *)
char                = "'" ( character | escape_sequence ) "'" ;

(* Strings *)
string              = normal_string | raw_string | raw_string_hash | auto_trimmed_string ;
normal_string       = '"' { string_char } '"' ;
raw_string          = 'r"' { any_char_except_quote } '"' ;
raw_string_hash     = 'r#"' { any_char_except_quote_hash } '"#' ;
auto_trimmed_string = '"""' newline { content_line } newline { whitespace } '"""' ;

(* DateTime *)
datetime            = 'd"' date_time_string '"' ;

(* Byte Data *)
hex_byte_data       = 'h"' { hex_byte } '"' ;

(* Compound Values *)
list                = "[" value { separator value } "]" ;
named_list          = "[" value ":" value { separator value ":" value } "]" ;
tuple               = "(" value { separator value } ")" ;
object              = "{" { identifier ":" value [separator] } "}" ;
enumeration         = identifier "::" identifier [ "(" value { separator value } ")" | "{" { identifier ":" value [separator] } "}" ] ;

(* Separators *)
separator           = "," | whitespace ;

(* Identifiers *)
identifier          = identifier_start { identifier_continue } ;
identifier_start    = "a"..."z" | "A"..."Z" | "_"
                    | U+00A0...U+D7FF | U+E000...U+10FFFF ;
identifier_continue = identifier_start | "0"..."9" ;

(* Comments *)
line_comment        = "//" { any_char_except_newline } newline ;
block_comment       = "/*" { any_char | block_comment } "*/" ;
```

### 7.6 Token Termination

Tokens in ASON are terminated by the following characters (also known as terminator characters):

- Whitespace: space (` `), tab (`\t`), carriage return (`\r`), line feed (`\n`)
- Comma: `,`
- Punctuation: `:`, `{`, `}`, `[`, `]`, `(`, `)`
- Comment start: `/` (when followed by `/` or `*`)
- End of file (EOF)

This means tokens such as numbers, identifiers, and keywords do not require explicit delimiters, they end when one of the above characters is encountered.

### 7.7 Processing Pipeline

An ASON parser typically processes the input in the following stages:

Stage 1. Character stream

The input is read as a stream of UTF-8 encoded characters with associated position information (index, line, and column).

Stage 2. Lexing (Tokenization)

The character stream is transformed into a token stream. During this stage:

- Whitespace, commas, and comments are consumed and discarded.
- Numeric literals are parsed (but sign tokens `+` and `-` are emitted as separate tokens).
- String literals (including raw strings and auto-trimmed strings) are fully processed.
- Identifiers are recognized, and keywords (`true`, `false`, `NaN`, `Inf`) are interpreted as their respective token types.
- The `::` sequence within an identifier produces an `Enumeration` token containing both the type name and variant name.

Stage 3. Normalization

The token stream is post-processed to merge sign tokens (`+` and `-`) with the following number token. During this stage:

- A `+` token followed by a number token is merged (the `+` is removed).
- A `-` token followed by a number token is merged (the number is negated).
- Overflow checks are performed for signed integer types.
- Applying signs to unsigned types or `NaN` is reported as an error.
- Bare number tokens (without a preceding sign) are checked for overflow in their signed range.

Stage 4. Parsing

The normalized token stream is parsed into an Abstract Syntax Tree (AST). During this stage, the parser recognizes compound structures (Objects, Lists, Tuples, Named Lists, Enumerations) and verifies structural correctness (matching brackets, required colons, non-empty tuples, etc.).

### 7.8 Error Handling

ASON errors are categorized as:

- Errors with position: Point to a specific character in the source (index, line number, column number). These typically occur during lexing when an unexpected character is encountered.
- Errors with range: Point to a span of characters in the source (start position and end position). These typically occur when a token or structure has an internal error (e.g., number overflow, invalid escape sequence).
- Unexpected end of document: Indicate that the input ended prematurely (e.g., an unclosed string, missing closing bracket).

### 7.9 File Extension and MIME Type

The file extension for ASON documents is `.ason`, and the MIME type is `application/ason`.

## 8 Linking

- [ASON source code on GitHub](https://github.com/hemashushu/ason)
- [ASON Rust library on Crates.io](https://crates.io/crates/ason)

## 9 License

This project is licensed under the MPL 2.0 License with additional terms. See the files [LICENSE](./LICENSE) and [LICENSE.additional](./LICENSE.additional)
