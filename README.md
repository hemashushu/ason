# ASON

_ASON_ is a data serialization format that evolved from JSON, featuring strong numeric typing and native support for enumeration types. With excellent readability and maintainability, ASON is well-suited for configuration files, data transfer, and data storage.

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->

- [1 Example](#1-example)
- [2 Improvements over JSON](#2-improvements-over-json)
- [3 Comparison with Other Data Serialization Formats](#3-comparison-with-other-data-serialization-formats)
- [4 Library and APIs](#4-library-and-apis)
  - [4.1 Deserialization and Serialization](#41-deserialization-and-serialization)
  - [4.2 Streaming Deserialization and Serialization](#42-streaming-deserialization-and-serialization)
- [5 Quick Reference](#5-quick-reference)
  - [5.1 Primitive Values](#51-primitive-values)
    - [5.1.1 Digit Separators](#511-digit-separators)
    - [5.1.2 Explicit Numeric Types](#512-explicit-numeric-types)
    - [5.1.3 Hexadecimal, Octal and Binary Integers](#513-hexadecimal-octal-and-binary-integers)
    - [5.1.4 Hexadecimal Floating-Point Numbers](#514-hexadecimal-floating-point-numbers)
    - [5.1.5 Special Floating-Point Numbers](#515-special-floating-point-numbers)
    - [5.1.6 String Representations](#516-string-representations)
  - [5.2 Compound Values](#52-compound-values)
    - [5.2.1 Lists](#521-lists)
    - [5.2.2 Tuples](#522-tuples)
    - [5.2.3 Objects](#523-objects)
    - [5.2.4 Maps](#524-maps)
    - [5.2.5 Enumerations](#525-enumerations)
    - [5.2.6 Types of Compound Values](#526-types-of-compound-values)
  - [5.3 Comments](#53-comments)
  - [5.4 ASON Documents](#54-ason-documents)
- [6 Mapping between Rust Data Types and ASON Types](#6-mapping-between-rust-data-types-and-ason-types)
  - [6.1 Structs](#61-structs)
  - [6.2 Vecs](#62-vecs)
  - [6.3 Tuples](#63-tuples)
  - [6.4 Array](#64-array)
  - [6.5 HashMaps](#65-hashmaps)
  - [6.6 Enums](#66-enums)
  - [6.7 Byte Data](#67-byte-data)
  - [6.8 DateTime](#68-datetime)
  - [6.9 Default Values](#69-default-values)
  - [6.10 Unsupported Data Types](#610-unsupported-data-types)
  - [6.11 ASONB Compatibility](#611-asonb-compatibility)
- [7 Links](#7-links)
- [8 License](#8-license)

<!-- /code_chunk_output -->

## 1 Example

Here is an example ASON document:

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
    map: [
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

## 2 Improvements over JSON

JSON has a simple syntax and has been around for decades, but it struggles to meet many modern needs. Several JSON variants have emerged to address its limitations, such as JSONC (which adds comments) and JSON5 (which allows trailing commas and unquoted object keys). However, these variants still cannot represent data accurately, because they inherit JSON's limited type system, which lacks fine-grained numeric types and domain-specific data types. ASON builds on JSON with the following improvements:

- **Explicit Numeric Types:** ASON numbers can be explicitly typed (e.g., `u8`, `i32`, `f32`, `f64`), which makes data representation more precise and rigorous. Integers can also be written in hexadecimal, octal, and binary.
- **More Primitive Data Types:** ASON adds `Char`, `DateTime`, and hexadecimal byte data types for representing common kinds of data.
- **Richer String Syntax:** ASON supports multi-line strings, concatenated strings, auto-trimmed strings, and raw strings.
- **More Precise Data Structures:** ASON distinguishes between `List` (elements of the same type) and `Tuple` (elements of different types), and provides `Map` (also called `Dictionary`) in addition to `Object` (also called `Struct`), which makes data structures clearer.
- **Native Enumeration Support:** ASON natively supports enumeration types (also known as _algebraic data types_, _tagged unions_, or _variants_), so complex data structures from high-level programming languages can be serialized seamlessly.
- **No Null Value:** ASON uses the `Option` enumeration to represent optional values, eliminating the error-prone `null` value.
- **Easy to Write:** ASON supports comments, unquoted object keys, trailing commas, and whitespace as an alternative to commas for separating elements. These features make documents easier to write and edit.

In addition to the text format, ASON has a binary counterpart called [ASONB (ASON Binary)](https://github.com/hemashushu/asonb). ASONB supports streaming, appendable root multi-chunk Strings, Byte Data, and Lists, and memory-mapped access. Incremental updates are limited, and performance benefits depend on the data and encoding choices.

ASON is designed to resemble JSON, so JSON users can learn and adopt it easily. Although ASON is not compatible with JSON, converting between the two is straightforward. ASON is also simple to parse, which makes it easy to implement in other languages and helps ensure its longevity as software ecosystems evolve.

## 3 Comparison with Other Data Serialization Formats

Many solid data serialization formats are available today, such as YAML, TOML, and XML. All of them are designed to be readable and writable by both humans and machines.

The differences between these formats are minor for small datasets, but become more pronounced as datasets grow and structures become more complex. For example, YAML's indentation-based syntax makes large documents error-prone to edit, TOML becomes cumbersome for deeply nested data, and XML's verbose syntax makes documents hard to read and write by hand. ASON, by contrast, is designed to stay simple, consistent, and expressive at any scale.

## 4 Library and APIs

The Rust [ason](https://github.com/hemashushu/ason) library is the reference implementation of ASON.

Add it to your project with:

```sh
cargo add ason
```

or add the following to your `Cargo.toml`:

```toml
[dependencies]
ason = "3.1"
serde = { version = "1.0", features = ["derive"] }
```

The `ason` crate provides [Serde](https://github.com/serde-rs/serde)-based APIs for serialization and deserialization as well as streaming APIs.

Unlike ASONB serialization, ASON serialization does not require a `Schema` trait or an embedded document schema. Deserialization requires an owned target (`DeserializeOwned`), so strings and byte payloads cannot be borrowed from the input as they can with ASONB's contiguous slice APIs.

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

This document contains an Object with three fields: `name`, `version`, and `dependencies`, where `dependencies` is a List of strings. It maps directly to the following Rust struct:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: Vec<String>,
}
```

The struct derives serde's `Serialize` and `Deserialize` traits, which enable serialization and deserialization.

Use `ason::de_from_str` to deserialize the ASON text into a `Package` instance:

```rust
// The ASON document shown above.
let document = "...";

// Deserialize the ASON document string into a `Package` struct instance.
let package: Package = ason::de_from_str(document).unwrap();

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

To serialize the `Package` value back to an ASON string, use `ason::ser_to_string`:

```rust
// Serialize the `Package` struct back into an ASON document string.
let document2 = ason::ser_to_string(&package).unwrap();
```

Single-document deserialization rejects tokens after the root value and checks for EOF, which can block on a live source. Use the streaming API for a live List. Serialization errors may leave partial output in the writer; writes are not transactional.

### 4.2 Streaming Deserialization and Serialization

For pipes, sockets, and standard I/O, ASON also supports streaming. In streaming mode, elements are written and read one at a time, instead of the whole document being built or loaded in memory at once. This is useful when the producer does not know the full list in advance, or when the consumer wants to start processing before the whole document has arrived.

Currently, the Serde streaming APIs support only documents whose root value is a List. ASON uses `[` and `]` rather than ASONB's multi-chunk binary framing. The closing `]` is required; EOF before it is an error. A stream reader returns an error once and then stops, and does not check for trailing content after `]`.

The streaming serializer lets you open a list writer, write elements as needed, and then close the list explicitly:

```rust
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let o1 = Object {
        id: 11,
        name: "foo".to_owned(),
    };

    let o2 = Object {
        id: 13,
        name: "bar".to_owned(),
    };

    let mut output = std::io::stdout().lock();
    let mut stream_writer = ason::list_stream_writer(&mut output);

    // Start the list before serializing elements.
    stream_writer.start_list().unwrap();

    // Serialize the first element.
    stream_writer.write(&o1).unwrap();

    // Serialize the second element.
    stream_writer.write(&o2).unwrap();

    // Serialize multiple elements at once is also supported.
    // stream_writer.writes(&[o1, o2]).unwrap();

    // End the list explicitly.
    stream_writer.end_list().unwrap();

    output.flush().unwrap();

    let mut stderr = std::io::stderr().lock();
    writeln!(&mut stderr, "Streaming serialization successful!").unwrap();
}
```

To write a batch, use `stream_writer.writes(&[o1, o2])` instead of the two separate `write` calls above. `writes` accepts a slice, so a vector can also be passed as `writes(&values)`. It writes elements in order, does nothing for an empty slice, and stops at the first serialization or I/O error. Like `write`, it may leave partial output on error and must be called between `start_list` and `end_list`.

The streaming deserializer reads the root list incrementally from a byte source (e.g., a pipe, a socket, or standard input). The typical flow is:

1. Wrap any `Read` source in a `ason::UTF8CharIterator`, and open a list reader on it.
2. Call `next()` repeatedly; each call deserializes the next element once it becomes available.
3. Stop when `next()` returns `None`, which indicates the end of the list.

For example, the following program reads the list of objects written by the serializer above:

```rust
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Object {
    id: i32,
    name: String,
}

fn main() {
    let mut input = std::io::stdin().lock();
    let mut char_iter = ason::UTF8CharIterator::new(&mut input);
    let mut stream_reader = ason::list_stream_reader(&mut char_iter).unwrap();

    // Deserialize the first element.
    let o1: Object = stream_reader.next().unwrap().unwrap();
    assert_eq!(
        o1,
        Object {
            id: 11,
            name: "foo".to_owned()
        }
    );

    // Deserialize the second element.
    let o2: Object = stream_reader.next().unwrap().unwrap();
    assert_eq!(
        o2,
        Object {
            id: 13,
            name: "bar".to_owned()
        }
    );

    // Read the remaining elements until the end of the list.
    while let Some(result) = stream_reader.next() {
        let _object: Object = result.unwrap();
        // ...
    }

    let mut stderr = std::io::stderr().lock();
    writeln!(&mut stderr, "Streaming deserialization successful!").unwrap();
}
```

The code above is available in the `examples` directory of the project. Build and run it with:

```sh
cargo build --examples
cd target/debug/examples
./stream-writer | ./stream-reader
```

## 5 Quick Reference

An ASON document is made up of values, which come in two kinds: primitive and compound. Primitive values are basic data types such as integers, strings, and booleans. Compound values, such as Lists and Objects, are made up of other values (primitive or compound).

For the complete syntax and parsing rules, see the [ASON Specification](docs/specification.md).

### 5.1 Primitive Values

ASON supports the following primitive value types:

**Numeric Values:**

- Integers: `123`, `+456`, `-789`
- Floating-point numbers: `3.142`, `+1.414`, `-1.732`
- Floating-point numbers in exponent notation: `2.998e10`, `6.674e-11`
- Special floating-point values: `NaN`, `+Inf` (or `Inf`), `-Inf`
- Hexadecimal integers: `0x41`, `+0x51`, `-0x61`
- Hexadecimal floating-point numbers: `0x1.4p3`, `0x1.921f_b6p1`
- Octal integers: `0o755`, `+0o600`, `-0o500`
- Binary integers: `0b1100`, `+0b1010`, `-0b1010_0100`

**Text Values:**

- Characters: `'a'`, `'文'`, `'😊'`
- Escape sequences: `'\r'`, `'\n'`, `'\t'`, `'\\'`
- Unicode escapes: `'\u{2d}'`, `'\u{6587}'`
- Strings: `"abc文字😊"`, `"foo\nbar"`
- Multi-line, concatenated, auto-trimmed, and raw strings: see section 5.1.6 below.

**Booleans, Dates, and Byte Data:**

- Booleans: `true`, `false`
- Date and time: `d"2024-03-16"`, `d"2024-03-16 16:30:50"`, `d"2024-03-16T16:30:50Z"`, `d"2024-03-16T16:30:50+08:00"`
- Hexadecimal byte data: `h"11 13 17 19"`

#### 5.1.1 Digit Separators

To improve readability, underscores can be inserted anywhere in a numeric literal (except within a prefix such as `0x`, a suffix such as `u32`, and in the special values `NaN` and `Inf`), including before and after the exponent marker `e`. For example: `123_456_789`, `6.626_070_e-34`

#### 5.1.2 Explicit Numeric Types

Numbers can be explicitly typed by appending a type suffix. For example: `65u8`, `3.14f32`

ASON supports the following numeric types: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`.

Underscores may also appear between the number and its type suffix for readability: `933_199_u32`, `6.626e-34_f32`

> If no explicit type is specified, integers default to `i32` and floating-point numbers default to `f64`.

#### 5.1.3 Hexadecimal, Octal and Binary Integers

In addition to decimal notation, ASON supports three other integer formats:

- Hexadecimal: prefix with `0x` (e.g., `0xFF`)
- Octal: prefix with `0o` (e.g., `0o755`)
- Binary: prefix with `0b` (e.g., `0b1010`)

These prefixes are case-insensitive (`0X`, `0O`, and `0B` are also valid). Note that leading zeros are not permitted in decimal integers (`0123` is invalid), but `0` by itself is valid.

#### 5.1.4 Hexadecimal Floating-Point Numbers

To write a hexadecimal floating-point number, use P notation: `0x1.921f_b6p1`, `0x1.4p3`. For details, see [P notation](https://en.wikipedia.org/wiki/Hexadecimal#Exponential_notation) or [Hexadecimal floating point literals](https://www.ibm.com/docs/en/xl-c-and-cpp-aix/16.1.0?topic=literals-floating-point).

Note that you cannot turn a hexadecimal integer into a floating-point number by appending a type suffix, because `f` is a valid hexadecimal digit. For example, `0x54321_f32` is parsed as the hexadecimal integer `0x5432_1f32`, not as a floating-point number of type `f32`.

#### 5.1.5 Special Floating-Point Numbers

ASON supports the special floating-point values:

- `NaN`
- `NaN_f32`, `NaN_f64`
- `Inf` (with optional sign: `+Inf`, `-Inf`)
- `Inf_f32`, `Inf_f64` (with optional sign: `+Inf_f32`, `-Inf_f32`, `+Inf_f64`, `-Inf_f64`)

They represent the results of certain mathematical operations (such as dividing by zero or taking the square root of a negative number), and are not valid in JSON.

Important notes:

- Do not confuse these with strings: `"NaN"` is a string, while `NaN` (unquoted) is a special floating-point value.
- `NaN` must not have a sign (`-NaN` and `+NaN` are invalid).

#### 5.1.6 String Representations

ASON strings can be written in several forms: normal strings, multi-line strings, concatenated strings, auto-trimmed strings, and raw strings.

##### 5.1.6.1 Normal Strings

Normal strings are enclosed in double quotation marks (`"`). For example:

```json5
{
    normal_string: "Hello, World!"
}
```

To include a double quotation mark (`"`) in a string, escape it with a backslash (`\"`). Similarly, to include a backslash (`\`), escape it with another backslash (`\\`).

##### 5.1.6.2 Multi-Line Strings

ASON strings can span multiple lines. For example:

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

In this form, all whitespace (including line breaks, tabs, and spaces) is part of the string content. For example, the content of the string above is:

```text
Planets in the Solar System:
        1. Mercury
        2. Venus
        3. Earth
        4. Mars
        5. Jupiter
        6. Saturn
        7. Uranus
        8. Neptune
```

Note that the leading spaces on each line are preserved. To remove them automatically, use concatenated strings (see section 5.1.6.3) or auto-trimmed strings (see section 5.1.6.4).

##### 5.1.6.3 Concatenated Strings

To improve readability, a long string can be split across multiple lines by adding a backslash (`\`) at the end of each line. The line break and the leading whitespace of the next line are removed, so the lines are joined together. For example:

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

This is equivalent to:

```json5
{
    concatenated_string: "My very educated mother just served us nine pizzas"
}
```

This form is especially useful for writing paragraphs or long sentences without introducing unwanted line breaks.

##### 5.1.6.4 Auto-Trimmed Strings

Auto-trimmed strings are another way to write multi-line strings. They automatically remove the common leading whitespace from every line. For example:

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

The leading whitespace is only for indentation and readability; it is not part of the string content.

In this example, the content lines have 8, 11, or 13 leading spaces. Since the minimum is 8, exactly 8 leading spaces are removed from each line. The resulting string is:

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
- The closing `"""` must be on a new line. It may be indented, but its indentation is ignored.
- Blank lines are ignored when calculating the minimum indentation.
- Escape sequences are not processed: backslashes and all other characters are kept as they are.

In short, the syntax of auto-trimmed strings is:

`"""\n...\n"""`

or

`"""\n...\n any_whitespace """`

where `...` represents the content lines. The two line breaks are mandatory and are not part of the string content.

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

##### 5.1.6.5 Raw Strings

Raw strings are enclosed in double quotation marks and prefixed with `r`. All characters within a raw string are treated literally.

They are useful for strings that contain special characters, because these characters do not need to be escaped. For example:

```json5
{
    raw_string: r"[a-z]\d+"
}
```

In the example above, the backslash is treated as a literal character, not as the start of an escape sequence. Therefore, `r"[a-z]\d+"` represents the regular expression `[a-z]\d+` exactly as written.

To include double quotation marks in a raw string, add one or more hash symbols (`#`) between the `r` and the opening quotation mark, and the same number after the closing quotation mark. For example:

```json5
{
    raw_string_with_hash: r#"He said, "Hello, World!""#
    raw_string_with_two_hashes: r##"The string "#" is a hash symbol."##
}
```

The number of hash symbols must be the same on both sides. A quotation mark followed by fewer hash symbols than the opening delimiter does not end the string; the quotation mark and those hash symbols are part of the string content. If a quotation mark is followed by more hash symbols than the opening delimiter, the extra hash symbols are treated as unexpected characters and cause a parsing error.

### 5.2 Compound Values

ASON supports the following compound value types: Lists, Tuples, Objects, Maps, and Enumerations.

#### 5.2.1 Lists

A List is an ordered collection of values of the same type. For example, this is a List of integers:

```json5
[11, 13, 17, 19]
```

And this is a List of strings:

```json5
["Alice", "Bob", "Carol", "Dan"]
```

All elements in a List must be of the same type. For example, the following List is invalid because it contains both integers and strings:

```json5
[11, 13, "Alice", "Bob"]    // Invalid List.
```

##### 5.2.1.1 Element Separators

The elements of a List can also be written on separate lines, each optionally followed by a comma:

```json5
[
    "Alice",
    "Bob",
    "Carol",
    "Dan",  // A trailing comma is allowed.
]
```

A comma after the last element (a trailing comma) is allowed. This makes it easier to reorder and add elements in a text editor.

Commas improve readability, but they are not mandatory. Therefore, the following List is also valid and is equivalent to the List above:

```json5
[
    "Alice"  // Commas can be omitted.
    "Bob"
    "Carol"
    "Dan"
]
```

Whitespace (such as spaces, tabs, and line breaks) can also separate elements, so the following List is valid too:

```json5
["Alice" "Bob" "Carol" "Dan"]
```

> In ASON, commas are optional and whitespace can be used as separators.

##### 5.2.1.2 Number of Elements

A List can contain any number of elements, including none. A List with no elements (an empty List) is valid:

```json5
[] // An empty List.
```

#### 5.2.2 Tuples

A Tuple is an ordered collection of a fixed number of values, which may be of different types. For example:

```json5
(11, "Alice", true)
```

As described above, commas are optional and whitespace can be used as separators, so the following Tuples are all equivalent to the one above:

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

Tuples differ from Lists in the following ways:

- The elements of a Tuple can be of different types, while the elements of a List must be of the same type.
- The number of elements in a Tuple is fixed, while the number of elements in a List is variable.
- Tuples are enclosed in parentheses (`(...)`), while Lists are enclosed in square brackets (`[...]`).

#### 5.2.3 Objects

An Object is a collection of key-value pairs, where each key is an identifier and each value can be of any type.

```json5
{
    name: "ason",
    version: "1.0.1",
    edition: "2021"
}
```

Keys are identifiers, which look like strings without quotation marks (`"`). An identifier must start with a letter (`a`-`z`, `A`-`Z`), an underscore (`_`), or a non-ASCII character (such as a CJK character), followed by any combination of these characters and digits (`0`-`9`). For example, `name`, `version`, `_edition`, and `foo_bar123` are all valid identifiers. See [Objects in the ASON Specification](docs/specification.md#44-objects) for the precise rules.

Key-value pairs in an Object can be separated by commas, whitespace (including line breaks), or both. The following Objects are all equivalent:

```json5
// Separated by commas.
{ name: "ason", version: "1.0.1", edition: "2021" }

// Separated by commas, with a trailing comma.
{ name: "ason", version: "1.0.1", edition: "2021",}

// Separated by spaces.
{ name: "ason" version: "1.0.1" edition: "2021" }

// Separated by line breaks.
{
    name: "ason"
    version: "1.0.1"
    edition: "2021"
}

// Separated by both commas and line breaks.
{
    name: "ason",
    version: "1.0.1",
    edition: "2021",
}
```

The values in an Object can be of any type, including primitive values and compound values. In practice, Objects often contain other Objects and Lists, for example:

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

> Objects are also called _Structs_ in some programming languages.

ASON Objects differ from JSON Objects in two ways: their keys are identifiers rather than strings, and their set of keys is fixed by the Object's type, whereas a JSON Object can have any number of key-value pairs.

#### 5.2.4 Maps

A Map is a special List in which each value is associated with a name. Such elements are called name-value pairs. The following Map consists of string-string pairs:

```json5
[
    "serde": "1.0"
    "serde_bytes": "0.11"
    "chrono": "0.4.38"
]
```

Names are usually strings, but other types are also allowed. For example, the following Map consists of integer-string pairs:

```json5
[
    5: "Perfect"
    4: "Good"
    3: "Fair"
    2: "Poor"
    1: "Try Again"
]
```

All names in a Map must be of the same type, and so must all values.

Maps differ from Objects in the following ways:

- The keys of an Object are identifiers, while the names in a Map can be of any type.
- The number of key-value pairs in an Object is fixed, while the number of name-value pairs in a Map is variable.
- Objects are enclosed in curly braces (`{...}`), while Maps are enclosed in square brackets (`[...]`).

> Maps are also called _Dictionaries_ in some programming languages.

ASON Maps with string names are similar to JSON Objects, except that they are enclosed in square brackets (`[...]`) instead of curly braces (`{...}`).

#### 5.2.5 Enumerations

An Enumeration is a data type that consists of a type name and a set of variants. Each variant can optionally carry a value. The following example shows an Enumeration type named `Option` with two variants, `None` and `Some`. `None` carries no value, while `Some` carries an integer:

```json5
Option::None
Option::Some(11)
```

A variant can carry a value of any type, such as a List, an Object, or a Tuple:

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

When a variant carries multiple values, it is more convenient to use the Object-like or Tuple-like forms, for example:

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

Note the difference between `Option::Some((1, "foo", true))`, which carries a single Tuple value, and `Color::RGB(255, 127, 63)`, which carries three values.

> Enumerations are also known as _algebraic data types_, _tagged unions_, or _variants_ in some programming languages.

#### 5.2.6 Types of Compound Values

Like primitive values, compound values have types. When an ASON document is deserialized into values of a programming language, the type of each compound value must match the expected type; otherwise, deserialization fails.

> Since all elements of a List must be of the same type, all compound values in a List must also have the same type.

##### 5.2.6.1 Lists

The type of a List is determined by the type of its elements. For example, a List of integers has the type `[i32]`, and a List of strings has the type `[String]`.

Since the number of elements in a List is variable, the type of a List does not depend on its length. For example, `[11, 13, 17]` and `[101, 103, 107, 109]` are both of type `[i32]`, even though the first List has 3 elements and the second has 4.

The type of an empty List is determined by the context in which it is used.

##### 5.2.6.2 Tuples

The type of a Tuple is determined by the type and order of its elements. For example, the Tuple `(11, "Alice", true)` has the type `(i32, String, bool)`, while the Tuple `("Bob", 3.14)` has the type `(String, f64)`.

##### 5.2.6.3 Objects

The type of an Object is determined by the keys and the corresponding value types. For example, the Object

```json5
{
    id: 123
    name: "Alice"
}
```

has the type `{ id: i32, name: String }`.

Some programming languages allow default values for missing fields, so an Object with missing fields can still match a type. For example, `{id: 123}` can be deserialized as type `{ id: i32, name: String }` if the `name` field has a default value.

However, `{id: 123, name: "Bob"}` and `{id: "001", name: "Alice"}` are not of the same type, because their `id` fields have different types.

##### 5.2.6.4 Maps

The type of a Map is determined by the types of its names and values. For example, the following Map:

```json5
[
    "red": 0xff0000
    "green": 0x00ff00
    "blue": 0x0000ff
]
```

has the type `[String: i32]`.

##### 5.2.6.5 Enumerations

The type of an Enumeration value is determined by its type name. For example, `Option::None` has the type `Option`, and `Shape::Rect{width: 200, height: 100}` has the type `Shape`. Therefore, different variants of the same Enumeration can appear in the same List.

### 5.3 Comments

Like JavaScript, C/C++, and Rust, ASON supports two kinds of comments: line comments and block comments. Comments are for human readers and are completely ignored by the parser.

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

Unlike JavaScript and C/C++ (but like Rust), ASON allows block comments to be nested. For example:

```rust
/*
    This is the first level.
    /*
        This is the second level.
    */
    This is the first level again.
*/
```

Nesting makes it easy to comment out a section that already contains block comments.

### 5.4 ASON Documents

An ASON document contains exactly one root value, which can be either a primitive value or a compound value. A typical ASON document is an Object or a List, but any type of value is allowed, even a single number, a string, or a Tuple. The following two are valid ASON documents:

```json5
// The root value is a Tuple.
(11, "Alice", true)
```

and

```json5
// The root value is a string.
"Hello World!"
```

The following two are invalid:

```json5
// There are two values at the root level.
(11, "Alice", true)
"Hello World!"
```

and

```json5
// There are three values at the root level.
11, "Alice", true
```

## 6 Mapping between Rust Data Types and ASON Types

ASON natively supports most Rust data types, including tuples, enums, and vectors. Because ASON numbers are strongly typed, numeric values keep their exact types during both serialization and deserialization.

The following Rust data types are supported:

- Signed and unsigned integers, from `i8`/`u8` to `i64`/`u64`
- Floating-point numbers: `f32` and `f64`
- `bool`
- `char`
- `String`
- `serde_bytes::Bytes`, `serde_bytes::ByteBuf`, `serde_bytes::ByteArray<N>`, and fields using `#[serde(with = "serde_bytes")]` (Byte Data, see section 6.7)
- `ason::date_time::DateTime` and chrono fields using `#[serde(with = "ason::date_time")]` (DateTime, see section 6.8)
- Arrays, such as `[i32; 4]` (Tuples by default; explicit mappings are described in section 6.4)
- `Vec`
- Tuples
- Structs
- `HashMap`
- Enums

`usize` and `isize` use `u64` and `i64`, respectively, following Serde's mapping. Slices serialize as Lists, and ownership wrappers such as `Box<T>` serialize their contents. `Option<T>` and `Result<T, E>` use Enumerations with their usual variant names.

See section 6.10 for unsupported Rust data types and section 6.11 for differences from ASONB.

### 6.1 Structs

Rust structs correspond to ASON Objects. The following is a struct named `User`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String
}
```

Here is an ASON document for an instance of `User`:

```json5
{
    id: 123
    name: "John"
}
```

Real-world data is often nested; for example, a struct may contain another struct. The following code shows a struct `User` that contains a child struct `Address`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    address: Address
}

#[derive(Serialize, Deserialize)]
struct Address {
    city: String,
    street: String
}
```

Here is an ASON document for an instance of the `User` struct above:

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

`Vec` corresponds to ASON List. The following code adds a field named `orders` to the struct `User` to store order numbers:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    orders: Vec<i32>
}
```

Here is an ASON document for an instance of `User`:

```json5
{
    id: 123
    name: "John"
    orders: [11, 13, 17, 19]
}
```

Fixed-length arrays have a different default mapping from `Vec<T>`; see section 6.4 for the default and explicit array mappings.

The elements of a vector can also be compound data, such as structs. The following code adds a field named `addresses` to the struct `User`:

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

Here is an ASON document for an instance of the `User` struct above:

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

### 6.3 Tuples

Rust tuples correspond to ASON Tuples.

For example, the following code creates a vector of tuples, where each tuple consists of an integer and a string:

```rust
let orders = vec![
    (11, String::from("ordered")),
    (13, String::from("shipped")),
    (17, String::from("delivered")),
    (19, String::from("cancelled"))
];
```

The corresponding ASON document for `orders` is:

```json5
[
    (11, "ordered")
    (13, "shipped")
    (17, "delivered")
    (19, "cancelled")
]
```

In some programming languages, tuples and vectors are not clearly distinguished, but in Rust they are completely different data types: a vector requires all its elements to be of the same type, while a tuple has a fixed number of elements, each with its own type. ASON's Tuple follows the same convention.

### 6.4 Array

Rust fixed-length arrays `[T; N]` have the following mappings:

| Field type                     | Default Serde mapping | `#[serde(with = "serde_bytes")]` | `#[serde(with = "ason::array")]` |
|--------------------------------|-----------------------|--------------------------------|---------------------------------|
| `[u8; N]`                      | Tuple of `u8` values  | Byte Data     | List of `u8` values |
| `[T; N]` where `T` is not `u8` | Tuple of `T` values   | Not supported | List of `T` values  |

The array mapping is:

- Default: Serde treats arrays as tuples, not sequences. For example, `[11, 13, 17, 19]` of type `[i32; 4]` becomes `(11 13 17 19)`, and `*b"abc"` of type `[u8; 3]` becomes `(97_u8 98_u8 99_u8)`. Deserialization expects a Tuple with exactly `N` elements. This is also ASONB's default array mapping. The Serde version used by this crate supplies default array implementations for lengths through 32; empty arrays cannot use this mapping because ASON does not support empty Tuples.

- With `serde_bytes`: Apply `#[serde(with = "serde_bytes")]` to a `[u8; N]` field to read and write Byte Data, such as `h"61 62 63"`. Deserialization requires exactly `N` bytes. For a root value, use `serde_bytes::ByteArray<N>`. Adding the dependency alone does not change an array's mapping, and this adapter does not apply to non-byte arrays.

- With `ason::array`: Apply `#[serde(with = "ason::array")]` to a `[T; N]` field to read and write a List. It works for any const-generic length, including zero and lengths greater than 32, provided `T` implements the required Serde traits. Deserialization requires exactly `N` elements; too few or too many elements, wrong element types, and Tuple or Byte Data inputs are errors. Byte arrays also become Lists under this adapter, not Byte Data.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Record {
    default_bytes: [u8; 3],

    #[serde(with = "serde_bytes")]
    byte_data: [u8; 3],

    #[serde(with = "ason::array")]
    byte_list: [u8; 3],

    default_array: [i32; 4],

    #[serde(with = "ason::array")]
    integer_list: [i32; 4],
}
```

For byte fields containing `b"abc"` and integer fields containing `[11, 13, 17, 19]`, the corresponding ASON document is:

```json5
{
    default_bytes: (97_u8 98_u8 99_u8)
    byte_data: h"61 62 63"
    byte_list: [97_u8 98_u8 99_u8]

    default_array: (11 13 17 19)
    integer_list: [11 13 17 19]
}
```

An adapted `[T; 0]` becomes `[]`, and a byte-adapted `[u8; 0]` becomes `h""`. `ason::array` changes only the annotated array's outer container; nested array elements keep their own Serde mappings. It does not change unannotated arrays or Rust tuples, and works with reader, writer, and streaming APIs.

### 6.5 HashMaps

Rust's `HashMap` corresponds to ASON Map. For example, the following is a `HashMap` with key type `String` and value type `i32`:

```rust
let mut m1 = HashMap::<String, i32>::new();
m1.insert("foo".to_owned(), 11);
m1.insert("bar".to_owned(), 22);
m1.insert("baz".to_owned(), 33);
```

The corresponding ASON document for `m1` is:

```json5
[
    "foo": 11
    "bar": 22
    "baz": 33
]
```

Note that the order of the entries depends on the iteration order of the `HashMap`, which is not guaranteed.

### 6.6 Enums

Rust enums correspond to ASON Enumerations. The following code defines an enum named `Color` with four variants: `Transparent`, `Grayscale`, `Rgb`, and `Hsl`.

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

Consider the following vector of `Color` values:

```rust
let colors = vec![
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

The corresponding ASON document for `colors` is:

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

### 6.7 Byte Data

`Vec<u8>` is serialized as a List of `u8`, not Byte Data. Use `serde_bytes::Bytes` for serialization of a byte slice, `serde_bytes::ByteBuf` for an owned value, or `#[serde(with = "serde_bytes")]` on a field to select Byte Data.

```rust
let data = serde_bytes::ByteBuf::from(vec![0x00, 0x11, 0xff]);
let text = ason::ser_to_string(&data).unwrap();
assert_eq!(text, r#"h"00 11 ff""#);
assert_eq!(ason::de_from_str::<serde_bytes::ByteBuf>(&text).unwrap(), data);
```

For struct fields, use the `serde_bytes` adapter. First add `serde_bytes` crate to your application's dependencies.

```toml
[dependencies]
serde_bytes = "0.11"
```

Then, in your Rust code, add

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Record {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}
```

Empty Byte Data is written as `h""`. Deserialization requires an owned target because the lexer owns the byte payload; it cannot borrow a byte slice from the input.

### 6.8 DateTime

Use `ason::date_time::DateTime` for a root value, or `#[serde(with = "ason::date_time")]` on a `chrono::DateTime<chrono::FixedOffset>` field. These explicit mappings follow the same usage pattern as ASONB and preserve the instant and numeric UTC offset.

First, add `chrono` to your application's dependencies; chrono's `serde` feature is not required by this adapter.

```toml
[dependencies]
chrono = "0.4"
```

Ordinary chrono serialization, with chrono's `serde` feature enabled, produces a regular String in both ASON and ASONB. Use the format-specific DateTime adapter to select DateTime instead.

```rust
let time = chrono::DateTime::parse_from_rfc3339("2024-03-16T16:30:50+08:00").unwrap();
let value = ason::date_time::DateTime(time);
let text = ason::ser_to_string(&value).unwrap();

assert_eq!(text, r#"d"2024-03-16T16:30:50+08:00""#);
assert_eq!(ason::de_from_str::<ason::date_time::DateTime>(&text).unwrap(), value);
```

For struct fields, use the `ason::date_time` adapter:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Record {
    #[serde(with = "ason::date_time")]
    time: chrono::DateTime<chrono::FixedOffset>,
}
```

Serialization writes a full date and time, using `Z` for UTC and `+HH:MM` or `-HH:MM` for other offsets. Deserialization also accepts ASON's date-only and timezone-omitted forms, filling omitted parts with midnight and UTC as appropriate. Regular String literals are not accepted as DateTime.

Unlike ASONB, ASON text does not support fractional seconds or offsets with second precision. Serialization rejects these values, and local years outside `0000` through `9999`, rather than silently losing precision. Whole leap seconds accepted by chrono can be represented as `:60`. The wrappers and adapters also work in nested values and with the reader, writer, and streaming APIs.

### 6.9 Default Values

Serde allows you to specify default values for missing struct fields, and ASON supports this feature. For example:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,

    name: String,

    #[serde(default = "default_age")]
    age: u8
}

fn default_age() -> u8 {
    18
}
```

In this example, the default value of the `age` field is provided by the `default_age` function. If the `age` field is missing from the ASON document, it is filled with the default value during deserialization. For example, the following ASON document is valid and can be deserialized into an instance of `User`:

```json5
{
    id: 123
    name: "John"
}
```

### 6.10 Unsupported Data Types

The following Rust data types are not supported:

- Unit (i.e., `()`)
- Unit structs, such as `struct Foo;`
- Newtype structs, such as `struct Width(u32);`
- Tuple structs, such as `struct RGB(u8, u8, u8);`

The dedicated `ason::date_time::DateTime` wrapper is an exception to the newtype limitation. Empty Tuples are also unsupported. Deserialization requires a concrete target type; Serde's `deserialize_any`, including the path used by untagged enums, is not supported.

### 6.11 ASONB Compatibility

The default numeric, List, Map, Tuple, Object, Enumeration, array, and explicit Byte Data mappings follow [ASONB's Rust type mappings](https://github.com/hemashushu/asonb#23-rust-type-mappings), but the libraries are not interchangeable:

- ASONB serialization requires schema information; ASON serialization does not. ASONB supports additional binary-only mappings for unit values, unit structs, tuple structs, newtype structs, and empty Tuples. These do not imply that every such binary value is representable in ASON text.
- ASON DateTime supports whole seconds and whole-minute offsets within its text range. ASONB additionally preserves nanoseconds and second-precision offsets, but rejects chrono leap-second values. ASON accepts whole leap seconds as `:60`; these cannot be passed unchanged to ASONB's DateTime adapter.
- `ason::array` explicitly selects List instead of the shared default Tuple mapping. When sharing such a struct with ASONB, its schema must describe the emitted List; the ASONB documentation requires `#[asonb(schema_with = "path")]` for custom adapters not covered by its schema derive.
- Serde default fields are an ASON deserialization feature. ASONB records fields in the document schema, so omitted fields must also match its schema rules; ASONB rejects value-dependent `skip_serializing_if` in its schema derive.
- ASONB-to-ASON conversion must reject values outside ASON's syntax or range unless an explicit conversion policy is supplied. Conversion preserves representable values, not comments, source formatting, or binary encoding choices. ASONB fixed-capacity slots require application-defined interpretation before conversion.

## 7 Links

- [ASON Specification](docs/specification.md)
- [ASONB Specification](https://github.com/hemashushu/asonb/blob/main/README.md)
- [ASON library](https://github.com/hemashushu/ason)
- [ASONB library](https://github.com/hemashushu/asonb)
- [ASON Converters (ason2b, ason2j)](https://github.com/hemashushu/ason-converter)
- [ASON Query (aq)](https://github.com/hemashushu/ason-query)

## 8 License

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0) with additional terms. See [LICENSE](./LICENSE) and [LICENSE.additional](./LICENSE.additional) for details.
