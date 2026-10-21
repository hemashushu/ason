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
  - [6.3 HashMaps](#63-hashmaps)
  - [6.4 Tuples](#64-tuples)
  - [6.5 Enums](#65-enums)
  - [6.6 Unsupported Data Types](#66-unsupported-data-types)
  - [6.7 Default Values](#67-default-values)
- [7 Specification](#7-specification)
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
    - [7.4.2 Maps](#742-maps)
    - [7.4.3 Tuples](#743-tuples)
    - [7.4.4 Objects](#744-objects)
    - [7.4.5 Enumerations](#745-enumerations)
  - [7.5 Grammar Summary](#75-grammar-summary)
  - [7.6 Token Termination](#76-token-termination)
  - [7.7 Processing Pipeline](#77-processing-pipeline)
  - [7.8 Error Handling](#78-error-handling)
  - [7.9 File Extension and MIME Type](#79-file-extension-and-mime-type)
- [8 Links](#8-links)
- [9 License](#9-license)

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

In addition to the text format, ASON has a binary counterpart called [ASONB (ASON Binary)](https://github.com/hemashushu/asonb). ASONB supports incremental updates, streaming, and random access, making it suitable for a wide range of applications, from simple data storage to complex data processing pipelines.

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
ason = "3.0"
```

The `ason` crate provides [Serde](https://github.com/serde-rs/serde)-based APIs for serialization and deserialization as well as streaming APIs.

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

### 4.2 Streaming Deserialization and Serialization

For pipes, sockets, and standard I/O, ASON also supports streaming. In streaming mode, elements are written and read one at a time, instead of the whole document being built or loaded in memory at once. This is useful when the producer does not know the full list in advance, or when the consumer wants to start processing before the whole document has arrived.

Currently, streaming is supported only for documents whose root value is a List.

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

    // End the list explicitly.
    stream_writer.end_list().unwrap();

    output.flush().unwrap();

    let mut stderr = std::io::stderr().lock();
    writeln!(&mut stderr, "Streaming serialization successful!").unwrap();
}
```

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
    while stream_reader.next().is_some() {
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

ASON supports the special floating-point values `NaN` and `Inf`. They represent the results of certain mathematical operations (such as dividing by zero or taking the square root of a negative number), and are not valid in JSON.

Important notes:

- Do not confuse these with strings: `"NaN"` is a string, while `NaN` (unquoted) is a special floating-point value.
- `NaN` must not have a sign (`-NaN` and `+NaN` are invalid).
- `Inf` can have an optional sign (`+Inf` or `-Inf`).
- Type suffixes are optional: `NaN_f32`, `NaN_f64`, `Inf_f32` and `Inf_f64`. A single underscore is required before the suffix, and no other underscores are allowed (`_Inf`, `I_nf`, and `Inf_` are invalid). The default type is `f64`.

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

`"""\n...\n optional_whitespace """`

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

Keys are identifiers, which look like strings without quotation marks (`"`). An identifier must start with a letter (`a`-`z`, `A`-`Z`), an underscore (`_`), or a non-ASCII character (such as a CJK character), followed by any combination of these characters and digits (`0`-`9`). For example, `name`, `version`, `_edition`, and `foo_bar123` are all valid identifiers. See section 7.4.4 for the precise rules.

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
    1: "Terrible"
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
- Arrays, such as `[i32; 4]` (serialized as Tuples, see section 6.6)
- `Vec`
- Tuples
- Structs
- `HashMap`
- Enums

See section 6.6 for the Rust data types that are not supported.

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

### 6.3 HashMaps

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

### 6.4 Tuples

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

### 6.5 Enums

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

### 6.6 Unsupported Data Types

The following Rust data types are not supported:

- Unit (i.e., `()`)
- Unit structs, such as `struct Foo;`
- Newtype structs, such as `struct Width(u32);`
- Tuple structs, such as `struct RGB(u8, u8, u8);`

The [serde data model](https://serde.rs/data-model.html) does not include a date-time type, so ASON DateTime values cannot be directly serialized from or deserialized into Rust's `chrono::DateTime`. Serializing a `chrono::DateTime` value produces a regular string.

In addition, serde treats fixed-length arrays such as `[i32; 4]` as tuples rather than vectors, so the Rust array `[11, 13, 17, 19]` (of type `[i32; 4]`) is serialized as the ASON Tuple `(11, 13, 17, 19)`.

### 6.7 Default Values

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

## 7 Specification

This section describes the ASON specification in detail. It is mainly intended for developers who want to implement ASON parsers and formatters in other programming languages. General users can skip it.

### 7.1 Document Structure

An ASON document consists of exactly one value at the root level. The root value can be any valid ASON value: a primitive value (Number, String, Boolean, etc.) or a compound value (Object, List, Tuple, etc.):

```json5
// A single Object as the root.
{id: 123, name: "Alice"}
```

```json5
// A single string as the root.
"Hello World!"
```

```json5
// A single number as the root.
42
```

A document with no value or more than one value at the root level is invalid:

```json5
// Two values at the root level (invalid).
(11, "Alice", true)
"Hello World!"
```

```json5
// Three values at the root level (invalid).
11, "Alice", true
```

### 7.2 Lexical Elements

#### 7.2.1 Encoding

ASON documents are encoded in UTF-8.

#### 7.2.2 Whitespace

The following characters are treated as whitespace and separate tokens. They are ignored by the parser (except within strings and characters):

| Character       | Code Point |
|-----------------|------------|
| Space           | `U+0020`   |
| Tab             | `U+0009`   |
| Carriage Return | `U+000D`   |
| Line Feed       | `U+000A`   |

The CRLF pair (`\r\n`, used on Windows) is treated as a single line break.

#### 7.2.3 Commas

Commas (`,`) serve as optional separators between elements of Lists, Tuples, Objects, and Maps. They carry no semantic meaning and are treated as whitespace by the parser. Trailing commas (after the last element) are permitted.

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

Inside a line comment, `/*` and `*/` have no special meaning. Similarly, inside a block comment, `//` has no special meaning (but `/*` and `*/` do, because block comments can be nested).

#### 7.2.5 Punctuation

The following punctuation characters are significant tokens:

| Token | Meaning                                            |
|-------|----------------------------------------------------|
| `{`   | Opening brace (Objects)                            |
| `}`   | Closing brace (Objects)                            |
| `[`   | Opening bracket (Lists and Maps)                   |
| `]`   | Closing bracket (Lists and Maps)                   |
| `:`   | Key-value separator (Objects and Maps)             |
| `(`   | Opening parenthesis (Tuples)                       |
| `)`   | Closing parenthesis (Tuples)                       |
| `+`   | Positive sign (Numbers)                            |
| `-`   | Negative sign (Numbers)                            |

Note that `::` (the separator between the type name and the variant name of an Enumeration) is not a standalone token. It is part of an Enumeration token, so no whitespace is allowed around it (see section 7.4.5).

### 7.3 Primitive Values

#### 7.3.1 Numbers

ASON supports integers and floating-point numbers. Every number has a specific data type, which is either specified by a type suffix or determined by default.

##### 7.3.1.1 Number Types

The supported number types are:

| Type  | Description                      | Width   | Range                           |
|-------|----------------------------------|---------|---------------------------------|
| `i8`  | Signed 8-bit integer             | 8 bits  | -128 to 127                     |
| `u8`  | Unsigned 8-bit integer           | 8 bits  | 0 to 255                        |
| `i16` | Signed 16-bit integer            | 16 bits | -32,768 to 32,767               |
| `u16` | Unsigned 16-bit integer          | 16 bits | 0 to 65,535                     |
| `i32` | Signed 32-bit integer            | 32 bits | -2,147,483,648 to 2,147,483,647 |
| `u32` | Unsigned 32-bit integer          | 32 bits | 0 to 4,294,967,295              |
| `i64` | Signed 64-bit integer            | 64 bits | -2^63 to 2^63 - 1               |
| `u64` | Unsigned 64-bit integer          | 64 bits | 0 to 2^64 - 1                   |
| `f32` | 32-bit floating-point (IEEE 754) | 32 bits | ±3.4028235e+38                  |
| `f64` | 64-bit floating-point (IEEE 754) | 64 bits | ±1.7976931348623157e+308        |

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

```text
hex_integer = "0" ("x" | "X") hex_digit { hex_digit }
hex_digit   = "0"..."9" | "a"..."f" | "A"..."F"
```

Examples: `0xFF`, `0x1A2B`, `0x00`

An empty hexadecimal number (e.g., `0x`) is invalid.

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
- The exponent must contain at least one digit. For example, `123e` and `1.5e+` are invalid.
- Multiple decimal points or exponent markers are not allowed.
- The default type is `f64`.

##### 7.3.1.7 Hexadecimal Floating-Point Numbers

Hexadecimal floating-point numbers use the [P notation](https://en.wikipedia.org/wiki/Hexadecimal#Exponential_notation) format (also known as hexadecimal exponential notation). The format is:

```text
hex_float = "0" ("x" | "X") hex_digits "." hex_digits ("p" | "P") [ "+" | "-" ] digits
```

The significand is written in hexadecimal, and the exponent (after `p`) is a decimal integer that specifies a power of 2.

Examples: `0x1.921fb6p1` (≈ π as f32), `0x1.5bf0a8b145769p+1` (≈ e as f64), `0x1.4p3` (= 10.0)

Rules:

- The exponent part (`p` or `P` followed by a decimal integer) is mandatory. A number like `0x1.23` without the `p` part is invalid.
- A number must not end with `.` or `p`. For example, `0x1.` and `0x1p` are both invalid.
- The default type is `f64`.
- Only `f32` and `f64` type suffixes are valid for hexadecimal floating-point numbers. Integer type suffixes (e.g., `i32`, `u32`) are not permitted.

> Note: You cannot turn a hexadecimal integer into a floating-point number by appending a type suffix, because `f` is a valid hexadecimal digit. For example, `0x54321_f32` is parsed as the hexadecimal integer `0x5432_1f32`, not as a floating-point number of type `f32`. Use P notation to write hexadecimal floating-point numbers.

##### 7.3.1.8 Digit Separators

Underscores (`_`) may be inserted between any two characters in a numeric literal to improve readability, and multiple underscores may appear in a row. They are ignored by the parser. However, underscores are not allowed within a prefix (e.g., `0_xFF` is invalid) or in the special values `NaN` and `Inf` (see section 7.3.1.11).

Examples: `123_456_789`, `0xFF_FF`, `0b1010_0101`, `6.626_070_e-34`

##### 7.3.1.9 Type Suffixes

An explicit type suffix may be appended to a number to specify its type. The suffix is one of the type names listed in section 7.3.1.1 (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`).

One or more underscores may appear between the number and its type suffix, but they are optional: `255_u8`, `3.14____f32`, `65u8`.

Rules for type suffixes in different number formats:

- Decimal integers and floating-point numbers: All type suffixes are valid.
- Hexadecimal integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. The `f32` and `f64` suffixes cannot be used because `f` is a valid hexadecimal digit.
- Hexadecimal floating-point numbers (P notation): Only `f32` and `f64` suffixes are valid.
- Octal and binary integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. Floating-point types are not supported for these formats.

It is an error if a number's value is out of the range of its type.

##### 7.3.1.10 Signed Numbers

Numbers can be preceded by a `+` (plus) or `-` (minus) sign.

Rules:

- The `+` sign is permitted for signed integer types and floating-point types. It does not change the value. A `+` sign in front of `NaN` is invalid.
- The `-` sign negates the number. It is permitted for signed integer types (`i8`, `i16`, `i32`, `i64`) and floating-point types (`f32`, `f64`). Applying `-` to unsigned types (`u8`, `u16`, `u32`, `u64`) is an error. A `-` sign in front of `NaN` is also invalid.
- Overflow is checked after the sign is applied. For example, `128_i8` is invalid because `128` exceeds `i8::MAX` (127), but `-128_i8` is valid because the range of `i8` includes -128.

##### 7.3.1.11 Special Floating-Point Values

ASON supports the special floating-point values `NaN` (Not a Number) and `Inf` (Infinity):

| Literal                              | Value                 |
|--------------------------------------|-----------------------|
| `NaN`, `NaN_f64`                     | f64 NaN               |
| `NaN_f32`                            | f32 NaN               |
| `Inf`, `+Inf`, `Inf_f64`, `+Inf_f64` | f64 positive infinity |
| `-Inf`, `-Inf_f64`                   | f64 negative infinity |
| `Inf_f32`, `+Inf_f32`                | f32 positive infinity |
| `-Inf_f32`                           | f32 negative infinity |

Rules:

- `NaN` must not have a sign. Both `+NaN` and `-NaN` are invalid.
- `Inf` can have a leading `+` or `-` sign.
- Only the `f32` and `f64` suffixes are valid for `NaN` and `Inf`. With any other suffix (e.g., `Inf_i32`, `NaN_i32`), the token is treated as a regular identifier.
- The literals listed in the table above are fixed spellings. The underscore before the suffix is mandatory, and no other underscores are allowed, so `Inff32`, `_Inf`, `I_nf`, `Inf_`, and `Inf__f32` are all invalid.
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

A character literal containing more than one character (e.g., `'ab'`) is also invalid. Note that some emojis consist of multiple Unicode code points, even though they are displayed as a single character. For example, the emoji 🤦‍♂️ consists of 4 code points: U+1F926, U+200D, U+2642, and U+FE0F. Therefore, this emoji enclosed in single quotes is an invalid character literal.

##### 7.3.3.1 Escape Sequences

The following escape sequences are supported in both character and string literals:

| Escape Sequence | Character                        |
|-----------------|----------------------------------|
| `\\`            | Backslash (`\`)                  |
| `\'`            | Single quote (`'`)               |
| `\"`            | Double quote (`"`)               |
| `\t`            | Horizontal tab (`U+0009`)        |
| `\n`            | Line feed (newline, `U+000A`)    |
| `\r`            | Carriage return (`U+000D`)       |
| `\0`            | Null character (`U+0000`)        |
| `\u{HHHHHH}`    | Unicode code point               |

Unicode escape sequences use the format `\u{H...}`, where `H...` is 1 to 6 hexadecimal digits representing a Unicode code point. The braces are mandatory. Examples: `\u{2d}` (hyphen `-`), `\u{6587}` (CJK character `文`).

Invalid Unicode code points (values greater than `U+10FFFF`, or surrogate code points from `U+D800` to `U+DFFF`) are errors.

The following escape sequences from other languages are not supported:

| Escape Sequence | Description                        |
|-----------------|------------------------------------|
| `\a`            | Alert (`U+0007`)                   |
| `\b`            | Backspace (`U+0008`)               |
| `\v`            | Vertical tab (`U+000B`)            |
| `\f`            | Form feed (`U+000C`)               |
| `\e`            | Escape character (`U+001B`)        |
| `\?`            | Question mark (`U+003F`)           |
| `\ooo`          | Octal character escapes            |
| `\xHH`          | Hexadecimal character escapes      |
| `\uHHHH`        | Unicode code point below `0x10000` |
| `\Uhhhhhhhh`    | Unicode code point                 |

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

This is equivalent to `"Line one\n    Line two\n    Line three"`. Note that all whitespace characters are preserved as they are.

##### 7.3.4.2 Concatenated Strings

A backslash (`\`) at the end of a line (immediately before the line break) joins the line with the next one. The line break and all leading whitespace of the next line are removed.

```json5
"The quick brown fox \
    jumps over \
    the lazy dog"
```

This is equivalent to `"The quick brown fox jumps over the lazy dog"`.

The backslash must be immediately followed by `\n` or `\r\n`. The leading whitespace (spaces and tabs) of the continuation line is removed.

##### 7.3.4.3 Auto-Trimmed Strings

Auto-trimmed strings are enclosed in triple double quotes (`"""`) and automatically remove the common leading whitespace from each line. The syntax is:

```text
auto_trimmed_string = '"""' newline { content_line } newline { whitespace } '"""'
```

Rules:

- The opening `"""` must be immediately followed by a line break (`\n` or `\r\n`).
- The closing `"""` must be on a new line. Whitespace before the closing `"""` is allowed but is not part of the content.
- The parser finds the minimum number of leading whitespace characters among all non-blank content lines, and removes that many leading characters from each line.
- Empty lines (lines containing only a line break) and blank lines (lines containing only whitespace) are ignored when calculating the minimum indentation.
- The line break immediately before the closing `"""` is not part of the string content.

Example:

```json5
"""
    Hello
      World
    Goodbye
"""
```

This produces the string `"Hello\n  World\nGoodbye"`. The minimum indentation is 4 spaces, so 4 spaces are removed from each line.

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

Note: Escape sequences are **not** processed within auto-trimmed strings. All characters (including backslashes) are preserved as they are (as in raw strings), except that the common leading whitespace is removed.

##### 7.3.4.4 Raw Strings

Raw strings are prefixed with `r` before the opening double quote: `r"..."`. Escape sequences are disabled within raw strings (backslashes and all other characters are treated literally).

```json5
r"^\d*(\.\d+)?$"
```

This is equivalent to `"^\\d*(\\.\\d+)?$"`.

The `r#"..."#` form uses hash delimiters, allowing the string to contain unescaped double quotes:

```json5
r#"<a href="https://hemashushu.github.io/" title="Home">Home Page</a>"#
```

This is equivalent to `"<a href=\"https://hemashushu.github.io/\" title=\"Home\">Home Page</a>"`.

More hash symbols can be used, such as `r##"..."##`, which allows the string to contain the sequence `"#`.

The number of hash symbols at the start and end of the string must match:

- A quotation mark followed by fewer hash symbols than the opening delimiter does not end the string; the quotation mark and those hash symbols are part of the string content.
- If a quotation mark is followed by more hash symbols than the opening delimiter, the extra hash symbols are treated as unexpected characters and cause a parsing error.

#### 7.3.5 DateTime

DateTime values are prefixed with `d` before a quoted date-time string:

```text
datetime = 'd"' date_time_string '"'
```

The supported formats are:

| Format                      | Example                        | Description                                |
|-----------------------------|--------------------------------|--------------------------------------------|
| `YYYY-MM-DD`                | `d"2024-03-16"`                | Date only (time defaults to 00:00:00, time zone defaults to UTC) |
| `YYYY-MM-DD HH:mm:ss`       | `d"2024-03-16 16:30:50"`       | Date and time (time zone defaults to UTC)  |
| `YYYY-MM-DDtHH:mm:ss`       | `d"2024-03-16t16:30:50"`       | Date and time with lowercase `t` separator |
| `YYYY-MM-DDTHH:mm:ssZ`      | `d"2024-03-16T16:30:50Z"`      | Date and time in UTC                       |
| `YYYY-MM-DDTHH:mm:ss±HH:MM` | `d"2024-03-16T16:30:50+08:00"` | Date and time with time zone offset        |

The date-time string is parsed according to [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339), with some relaxations. The date and time are separated by `T`, `t`, or a space. A time zone designator can be appended: `Z` or `z` for [Coordinated Universal Time (UTC)](https://en.wikipedia.org/wiki/Coordinated_Universal_Time) (equivalent to `+00:00`), or an offset such as `+08:00`. If no time zone is specified, UTC is assumed.

Characters allowed within the date-time string: digits (`0`-`9`), `-`, `:`, space (` `), `t`, `T`, `z`, `Z`, `+`.

#### 7.3.6 Hexadecimal Byte Data

Hexadecimal byte data is a sequence of bytes, each written as two hexadecimal digits, prefixed with `h`:

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
[
    h"48 65 6C 6C 6F 00",
    h"",
]
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

All elements in a List must be of the same type. For example, a List of integers and a List of strings are valid, but a List that mixes integers and strings is invalid.

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

#### 7.4.2 Maps

A Map (also called a Dictionary) is an ordered sequence of name-value pairs enclosed in square brackets. It is distinguished from a regular List by the colon (`:`) after the first element.

```text
map = "[" { name_value_pair [","] } "]"
name_value_pair = value ":" value
```

The names (on the left of `:`) can be of any type (strings, numbers, etc.), but all names must be of the same type. Similarly, all values must be of the same type.

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

An empty Map is written as an empty List `[]`.

#### 7.4.3 Tuples

A Tuple is an ordered sequence of one or more values enclosed in parentheses (`(` and `)`).

```text
tuple = "(" value { [","] value } [","] ")"
```

Unlike Lists, Tuples can contain elements of different types. The number and types of the elements define the Tuple's type.

An empty Tuple (`()`) is **not** valid.

Elements can be separated by commas, whitespace, or both. Trailing commas are permitted.

Examples:

```json5
(11, "Alice", true)
(3.14, 42_u8)
```

#### 7.4.4 Objects

An Object is an ordered sequence of zero or more key-value pairs enclosed in curly braces (`{` and `}`).

```text
object = "{" { key_value_pair [","] } "}"
key_value_pair = identifier ":" value
```

Keys are identifiers (unquoted). They must **not** be enclosed in double quotes (unlike JSON). If a quoted string is used as a key, it is a parse error.

An identifier follows these rules:

- Must start with a letter (`a`-`z`, `A`-`Z`), an underscore (`_`), or a Unicode character in the range U+00A0 to U+D7FF or U+E000 to U+10FFFF (which includes CJK characters, Greek letters, emoji, etc.).
- May continue with letters, digits (`0`-`9`), underscores, or Unicode characters in the same ranges.
- Must not contain `::`. An identifier containing `::` becomes an Enumeration token (see section 7.4.5).

An empty Object (`{}`) is valid in ASON, even though some programming languages do not support empty structs. When an empty Object is deserialized into a struct, every field takes its default value.

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

An Enumeration value consists of a type name and a variant name, separated by `::`. It can optionally carry associated data.

```text
enumeration = type_name "::" variant_name [ variant_body ]
variant_body = "(" value { [","] value } [","] ")"      // single-value or tuple-like
             | "{" { key_value_pair [","] } "}"         // object-like
```

Both the type name and the variant name follow the rules for identifiers. No whitespace is allowed around `::`.

There are four forms of Enumeration values:

- Variant without associated data:

```json5
Option::None
Color::Red
```

- Single-value variant, with one value in parentheses:

```json5
Option::Some(123)
Option::Some("hello")
```

- Tuple-like variant, with multiple values in parentheses:

```json5
Color::RGB(255, 127, 63)
```

- Object-like variant, with key-value pairs in curly braces:

```json5
Shape::Rect{width: 200, height: 100}
```

A variant with empty parentheses (`Option::Some()`) is **not** valid. If a variant has parentheses, they must contain at least one value.

> Note: Whether a variant is a single-value variant (e.g., `Option::Some(123)`) or a Tuple-like variant (e.g., `Color::RGB(255, 127, 63)`) depends on the number of values in the parentheses: exactly one value makes it a single-value variant, and two or more values make it a Tuple-like variant.

### 7.5 Grammar Summary

The following is a summary of the ASON grammar in EBNF-like notation. The rules for tokens (numbers, strings, identifiers, etc.) describe exact character sequences. Whitespace, commas, and comments can appear between any two tokens and are discarded by the lexer (see section 7.2), so they are omitted from the rules for compound values.

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
                    | map
                    | tuple
                    | object ;

(* Numbers *)
(* A sign is also not allowed in front of unsigned integer types (checked after parsing). *)
number              = [ "+" | "-" ] ( decimal_number
                                    | hex_integer
                                    | hex_float
                                    | octal_integer
                                    | binary_integer
                                    | infinity )
                    | nan ;

(* Underscores may appear between any two characters of a number, except within a prefix
   and in "NaN" and "Inf", where only a single "_" before the type suffix is allowed. *)
sep                 = { "_" } ;

decimal_number      = integer_part [ sep "." sep digits ] [ sep exponent ]
                      [ sep ( int_type | float_type ) ] ;
(* Leading zeros are not allowed. *)
integer_part        = "0" | non_zero_digit { sep digit } ;
exponent            = ( "e" | "E" ) sep [ ( "+" | "-" ) sep ] digits ;

hex_integer         = "0" ( "x" | "X" ) sep hex_digits [ sep int_type ] ;
hex_float           = "0" ( "x" | "X" ) sep hex_digits sep "." sep hex_digits
                      sep ( "p" | "P" ) sep [ ( "+" | "-" ) sep ] digits
                      [ sep float_type ] ;
octal_integer       = "0" ( "o" | "O" ) sep octal_digits [ sep int_type ] ;
binary_integer      = "0" ( "b" | "B" ) sep binary_digits [ sep int_type ] ;

(* Fixed spellings; no other underscores are allowed. *)
infinity            = "Inf" | "Inf_f32" | "Inf_f64" ;
nan                 = "NaN" | "NaN_f32" | "NaN_f64" ;

int_type            = "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" ;
float_type          = "f32" | "f64" ;

digits              = digit { sep digit } ;
hex_digits          = hex_digit { sep hex_digit } ;
octal_digits        = octal_digit { sep octal_digit } ;
binary_digits       = binary_digit { sep binary_digit } ;

non_zero_digit      = "1"..."9" ;
digit               = "0"..."9" ;
hex_digit           = "0"..."9" | "a"..."f" | "A"..."F" ;
octal_digit         = "0"..."7" ;
binary_digit        = "0" | "1" ;

(* Boolean *)
boolean             = "true" | "false" ;

(* Character *)
(* Exactly one Unicode scalar value, U+0000...U+D7FF or U+E000...U+10FFFF. *)
char                = "'" ( any_char_except_quote_and_backslash | escape_sequence ) "'" ;
escape_sequence     = "\" ( "\" | "'" | '"' | "t" | "n" | "r" | "0" | unicode_escape ) ;
(* 1 to 6 hex digits; surrogates and values above U+10FFFF are invalid. *)
unicode_escape      = "u{" hex_digit { hex_digit } "}" ;

(* Strings *)
string              = normal_string
                    | raw_string
                    | raw_string_hash
                    | auto_trimmed_string ;
normal_string       = '"' { string_char } '"' ;
string_char         = any_char_except_double_quote_and_backslash
                    | escape_sequence
                    | line_continuation ;
(* Concatenated strings: the backslash, line break, and following indentation are removed. *)
line_continuation   = "\" newline { " " | "\t" } ;
raw_string          = 'r"' { any_char_except_double_quote } '"' ;
(* The content ends at the first '"' followed by the same number of hashes. *)
raw_string_hash     = "r" hashes '"' { any_char } '"' hashes ;
hashes              = "#" { "#" } ;
(* Escape sequences are not processed in auto-trimmed strings. *)
auto_trimmed_string = '"""' newline { content_line } newline { " " | "\t" } '"""' ;
content_line        = { any_char_except_newline } newline ;

(* DateTime *)
datetime            = 'd"' date [ ( "T" | "t" | " " ) time [ time_zone ] ] '"' ;
date                = digit digit digit digit "-" digit digit "-" digit digit ;
time                = digit digit ":" digit digit ":" digit digit ;
(* If the time zone is omitted, UTC is assumed. *)
time_zone           = "Z" | "z" | ( "+" | "-" ) digit digit ":" digit digit ;

(* Byte Data *)
hex_byte_data       = 'h"' { whitespace }
                      [ hex_byte { whitespace { whitespace } hex_byte } { whitespace } ]
                      '"' ;
hex_byte            = hex_digit hex_digit ;

(* Compound Values *)
list                = "[" { value } "]" ;
map                 = "[" name_value_pair { name_value_pair } "]" ;
name_value_pair     = value ":" value ;
tuple               = "(" value { value } ")" ;
object              = "{" { key_value_pair } "}" ;
key_value_pair      = identifier ":" value ;
(* No whitespace is allowed around "::". *)
enumeration         = identifier "::" identifier
                      [ "(" value { value } ")" | "{" { key_value_pair } "}" ] ;

(* Identifiers *)
identifier          = identifier_start { identifier_continue } ;
identifier_start    = "a"..."z" | "A"..."Z" | "_"
                    | U+00A0...U+D7FF | U+E000...U+10FFFF ;
identifier_continue = identifier_start | "0"..."9" ;

(* Whitespace *)
whitespace          = " " | "\t" | "\r" | "\n" ;
newline             = "\n" | "\r\n" ;

(* Comments *)
line_comment        = "//" { any_char_except_newline } ( newline | EOF ) ;
(* Block comments can be nested. *)
block_comment       = "/*" { any_char | block_comment } "*/" ;
```

### 7.6 Token Termination

The following characters (called terminators) end a token:

- Whitespace: space (` `), tab (`\t`), carriage return (`\r`), line feed (`\n`)
- Comma: `,`
- Punctuation: `:`, `{`, `}`, `[`, `]`, `(`, `)`
- Comment start: `/` (when followed by `/` or `*`)
- End of file (`EOF`)

This means that tokens such as numbers, identifiers, and keywords do not need explicit delimiters; they end when one of the above characters is encountered.

### 7.7 Processing Pipeline

An ASON parser typically processes the input in the following stages:

**Stage 1: Character Stream**

The input is decoded from UTF-8 into a stream of characters, each with its position (index, line, and column).

**Stage 2: Lexing (Tokenization)**

The character stream is converted into a token stream. During this stage:

- Whitespace, commas, and comments are consumed and discarded.
- Numeric literals are parsed (but the signs `+` and `-` are emitted as separate tokens).
- String literals (including raw strings and auto-trimmed strings) are fully processed.
- Identifiers are recognized, and keywords (`true`, `false`, `NaN`, `Inf`) are converted to their corresponding token types.
- An identifier containing `::` produces an `Enumeration` token that contains both the type name and the variant name.

**Stage 3: Normalization**

The token stream is post-processed to merge each sign token (`+` or `-`) with the number token that follows it. During this stage:

- A `+` token followed by a number token is merged (the `+` is removed).
- A `-` token followed by a number token is merged (the number is negated).
- Overflow is checked for signed integer types.
- Applying a sign to an unsigned type or to `NaN` is reported as an error.
- Number tokens without a sign are checked for overflow against the maximum value of their type (e.g., `128_i8` is an error).

**Stage 4: Parsing**

The normalized token stream is parsed into an Abstract Syntax Tree (AST). During this stage, the parser recognizes compound structures (Objects, Lists, Tuples, Maps, and Enumerations) and verifies that they are well-formed (matching brackets, required colons, non-empty Tuples, etc.).

### 7.8 Error Handling

ASON errors fall into three categories:

- Errors with a position: Point to a specific character in the source (index, line number, and column number). These typically occur during lexing when an unexpected character is encountered.
- Errors with a range: Point to a span of characters in the source (start position and end position). These typically occur when a token or structure contains an error (e.g., number overflow or an invalid escape sequence).
- Unexpected end of document: Indicates that the input ended prematurely (e.g., an unclosed string or a missing closing bracket).

### 7.9 File Extension and MIME Type

The file extension for ASON documents is `.ason`, and the MIME type is `application/ason`.

## 8 Links

- [ASON Specification and library](https://github.com/hemashushu/ason)
- [ASONB Specification and library](https://github.com/hemashushu/asonb)

## 9 License

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0) with additional terms. See [LICENSE](./LICENSE) and [LICENSE.additional](./LICENSE.additional) for details.
