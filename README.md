# ASON

_ASON_ is a data serialization format that evolved from JSON, featuring strong numeric typing and native support for variant types. With excellent readability and maintainability, ASON is well-suited for configuration files, data transfer, and data storage.

**Table of Content**

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [1. ASON Example](#1-ason-example)
- [2. Comparison of Common Data Serialization Formats](#2-comparison-of-common-data-serialization-formats)
- [3. What improvements does ASON bring over JSON?](#3-what-improvements-does-ason-bring-over-json)
- [4 Library and APIs](#4-library-and-apis)
  - [4.1 Serialization and Deserialization](#41-serialization-and-deserialization)
  - [4.2 Parser and Printer](#42-parser-and-printer)
- [5 ASON Quick Reference](#5-ason-quick-reference)
  - [5.1 Primitive Values](#51-primitive-values)
    - [5.1.1 Digit Separators](#511-digit-separators)
    - [5.1.2 Explicit Numeric Types](#512-explicit-numeric-types)
    - [5.1.3 Hexadecimal, Octal and Binary Integers](#513-hexadecimal-octal-and-binary-integers)
    - [5.1.4 Hexadecimal Floating-Point Numbers](#514-hexadecimal-floating-point-numbers)
    - [5.1.5 Special Floating-Point Numbers](#515-special-floating-point-numbers)
    - [5.1.6 String Presentation](#516-string-presentation)
      - [5.1.6.1 Multi-Line Strings](#5161-multi-line-strings)
        - [5.1.6.2 Concatenated Strings](#5162-concatenated-strings)
        - [5.1.6.3 Auto-Trimmed Strings](#5163-auto-trimmed-strings)
  - [5.2 Compound Values](#52-compound-values)
    - [5.2.1 Objects](#521-objects)
    - [5.2.2 Lists](#522-lists)
    - [5.2.3 Named Lists](#523-named-lists)
    - [5.2.4 Tuples](#524-tuples)
    - [5.2.5 Variants](#525-variants)
  - [5.3 Comments](#53-comments)
  - [5.4 Documents](#54-documents)
- [6 Mapping between ASON and Rust Data Types](#6-mapping-between-ason-and-rust-data-types)
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
    variant: Option::None
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

The differences between these formats are minor for small datasets, but become more pronounced as datasets grow or structures become more complex. For example, YAML's indentation-based syntax can cause errors in large documents, and TOML's limited support for complex structures can make representing hierarchical data cumbersome. JSON, by contrast, is designed to stay simple, consistent, and expressive at any scale.

For developers, JSON offers additional advantages:

- You don't have to learn an entirely new syntax: JSON closely resembles JavaScript object literals.
- Implementing a parser is straightforward, which helps ensure longevity and adaptability across evolving software ecosystems.

## 3. What improvements does ASON bring over JSON?

JSON has a simple syntax and has been around for decades, but it struggles to meet diverse modern needs. Many JSON variants have emerged to address its limitations—such as JSONC (which adds comments) and JSON5 (which allows trailing commas and unquoted object keys). However, these variants still cannot represent data accurately due to limitations like the lack of strong typing. ASON takes a significant step forward based on JSON with the following improvements:

- **Explicit Numeric Types:** ASON numbers can be explicitly typed (e.g., `u8`, `i32`, `f32`, `f64`) ensuring more precise and rigirous data representation. Additionally, integers can be represented in hexadecimal, octal, and binary formats.
- **New Data Types:** New data types such as `Char`, `DateTime`, and `ByteData` to better represent common data types.
- **More string formats:** "Multi-line strings", "Concatenate strings", "Raw strings", and "Auto-trimmed strings" are added to enhance string representation.
- **Separate List and Tuple Types:** ASON distinguishes between `List` (homogeneous elements) and `Tuple` (heterogeneous elements), enhancing data structure clarity.
- **Separate Named-List and Object Types:** ASON introduces `Named-List` (also called `Map`) alongside `Object` (also called `Struct`), enhancing data structure clarity in further.
- **Native Variant Type Support:** ASON natively supports variant types (also known as _algebraic types_ or _enumerations_). This enables seamless serialization of complex data structures from high-level programming languages.
- **Eliminating the Null Value:** ASON uses the `Option` variant to represent optional values, eliminating the error-prone `null` value and `undefined`.
- **Familiar to JSON Users:** ASON is designed to resemble JSON, making it easy for JSON users to learn and adopt.
- **Simple and Consistent:** ASON supports comments, unquoted object field names, trailing commas, and whitespace-separated elements (in addition to commas). These features enhance writing fluency.

In addition to the text format, ASON provides a binary format called _ASONB_ (ASON Binary) for efficient data storage and transmission, supporting incremental storage, memory-mapped file access, and fast random access.

It is worth noting that ASON is not compatible with JSON, but conversion between ASON and JSON is straightforward, and implementing an ASON parser is also simple.

## 4 Library and APIs

The Rust [ason](https://github.com/hemashushu/ason) library provides AST (Abstract Syntax Tree) level ASON access, and [serde_ason](https://github.com/hemashushu/serde_ason) provides [serde](https://github.com/serde-rs/serde) based for serialization and deserialization.

In general, it is recommended to use the serde API since it is simple enough to meet most needs.

### 4.1 Serialization and Deserialization

Consider the following ASON document:

```json5
{
    name: "foo"
    version: "0.1.0"
    dependencies: [
        "registry.domain/user/random@1.0.1"
        "registry.domain/user/regex@2.0.0"
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

The following code shows how to use the serde API `ason::from_str` for deserializing the ASON document into a Rust struct instance:

```rust
let text = "..."; // The above ASON document
let package = from_str::<Package>(text).unwrap();
```

And serialize a Rust struct instance to string with `ason::to_string` function:

```rust
let package = Package{
    name: String::new("foo"),
    version: String::new("0.1.0"),
    dependencies: vec![
        String::new("registry.domain/user/random@1.0.1"),
        String::new("registry.domain/user/regex@2.0.0"),
    ],
};
let text = to_string(&package);
```

### 4.2 Parser and Printer

The `ason::parse_from_str` function is used to parse the ASON document into AST:

```rust
let text = "..."; // The above ASON document
let node = parse_from_str(text).unwrap();

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
                AsonNode::String(String::from("registry.domain/user/random@1.0.1")),
                AsonNode::String(String::from("registry.domain/user/regex@2.0.0"))
            ]))
        }
    ])
);
```

And, the function `ason::print_to_string` is used to format the AST into text:

```rust
let text = print_to_string(&node);
// The `text` should be resemble the above ASON document
```

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

ASON supports the following compound value types: Objects, Lists, Named Lists, Tuples and Variants.

#### 5.2.1 Objects

An _Object_ can contain multiple values, each with a name called a _key_. The keys are _identifiers_ which are similar to strings but without quotation marks (`"`). A combination of a key and a value is called a _key-value pair_.

In other words, an Object is a collection of key-value pairs. For example:

```json5
{
    name: "ason",
    version: "1.0.1",
    edition: "2021"
}
```

Key-value pairs are separated by commas (the last key-value pair can also be followed by a comma) or whitespace (such as spaces, tabs, and line breaks). Thus the followings Objects are all identical:

```json5
// separated by commas
{ name: "ason", version: "1.0.1", edition: "2021" }

// separated by whitespace
{ name: "ason" version: "1.0.1" edition: "2021" }

// separated by line breaks
{
    name: "ason"
    version: "1.0.1"
    edition: "2021"
}

// separated by commas and line breaks
{
    name: "ason",
    version: "1.0.1",
    edition: "2021",
}
```

A comma at the end of the last key-value pair (which is called a trailing comma) is allowed in ASON, this feature is primarily intended to make it easy to reorder key-value pairs when editing multi-line objects.

> In ASON, commas are optional and whitespace can be used as separators.

The values within an Object can be any type, including primitive values (such as numbers, strings, dates) and compound values (such as Lists, Objects, Tuples). In the real world, an Object usually contains other Objects, for example:

```json5
{
    name: "ason"
    version: "1.0.1"
    edition: "2021"
    dependencies: {
        serde: "1.0"
        chrono: "0.4"
    }
    dev_dependencies: {
        pretty_assertions: "1.4"
    }
}
```

#### 5.2.2 Lists

A List is a collection of values of the same data type, for example:

```json5
[11, 13, 17, 19]
```

Similar to objects, the elements in a List can also be written on separate lines, with optional commas at the end of each line, and a comma is allowed at the end of the last element. For example:

```json5
[
    "Alice",
    "Bob",
    "Carol",
    "Dan",  // Note that ths comma is allowed.
]
```

and

```json5
[
    "Alice"  // Note that commas can be omitted.
    "Bob"
    "Carol"
    "Dan"
]
```

The elements in List can be of any data type, but all the elements in a List must be of the same type. For instance, the following List is invalid:

```json5
// invalid list due to inconsistent data types of elements
[11, 13, "Alice", "Bob"]
```

If the elements in a List are Objects, then the keys in each object, as well as the data type of the corresponding values, must be consistent. In other words, the type of object is determined by the type of all key-value pairs, and the type of key-value pair is determined by the key name and data type of the value. For example, the following List is valid:

```json5
[
    {
        id: 123
        name: "Alice"
    }
    {
        id: 456
        name: "Bob"
    }
]
```

While the following List is invalid:

```json5
[
    {
        id: 123
        name: "Alice"
    }
    {
        id: 456
        name: 'A'   // The data type of the value is not consistent.
    }
    {
        id: 789
        addr: "Green St." // The key name is not consistent.
    }
]
```

If the elements in a List are Lists, then the data type of the elements in each sub-list must be the same. In other words, the type of List is determined by the data type of its elements. But the number of elements is irrelevant, for instance, the following list is valid:

```json5
[
    [11, 13, 17] // The length of this list is 3.
    [101, 103, 107, 109] // A list of length 4 is Ok.
    [211, 223] // This list has length 2 is also Ok.
]
```

In the example above, although the length of each sub-list is different, since the type of a List is determined ONLY by the type of its elements, the types of these sub-lists are asserted to be the same, and therefore it is a valid List.

#### 5.2.3 Named Lists

TODO

<!-- A Map is a list composed of one or more key-value pairs. In appearance, a Map is similar to an Object, but the keys of items in a Map are typically strings or numbers (primitive data types), rather than identifiers. Additionally, a Map is a special kind of list, so it is enclosed in square brackets (`[...]`) instead of curly braces (`{...}`). -->

```json5
[
    "serde": "1.0"
    "serde_bytes": "0.11"
    "chrono": "0.4.38"
]
```

#### 5.2.4 Tuples

A Tuple can be considered as an Object that omits the keys, for example:

```json5
(11, "Alice", true)
```

Tuples are similar in appearance to Lists, but Tuples do not require the data types of each element to be consistent. Secondly, both the data type and number of the elements are part of the type of Tuple, for example `("Alice", "Bob")` and `("Alice", "Bob", "Carol")` are different types of Tuples because they don't have the same number of elements.

Similar to Objects and Lists, the elements of a Tuple can also be written on separate lines, with optional commas at the end of each line, and there can be a comma at the end of the last element. For example:

```json5
(
    "Alice",
    11,
    true, // Note that ths comma is allowed.
)
```

and

```json5
(
    "Alice" // Note that commas can be omitted.
    11
    true
)
```

#### 5.2.5 Variants

Variants also called enumeration. A Variant is a data type that can have multiple named members, and each member can optionally carry a value. Variants are useful for representing data that can take on different forms.

A Variant consists of three parts: the Variant type name, the Variant member name, and the optional member value. For example:

```json5
// Variant without value.
Option::None
```

and

```json5
// Variant with a value.
Option::Some(11)
```

In the two Variants in the above example, "Option" is the Variant type name, "None" and "Some" are the Variant member names, and "11" is the Variant member value.

The types are the same as long as the Variant type names are the same. For example, `Color::Red` and `Color::Green` are of the same type, while `Option::None` and `Color::Red` are of different types.

If a Variant member carries a value, then the type of the value is also part of the type of the Variant member. For example, `Option::Some(11)` and `Option::Some(13)` are of the same types, but `Option::Some(11)` and `Option::Some("John")` are of different types.

Therefore, the following List is valid because all elements have the same Variant type name and the member `Some` has the same type:

```json5
[
    Option::None
    Option::Some(11)
    Option::None
    Option::Some(13)
]
```

However, the following List is invalid, although the variant type names of all the elements are consistent, the type of the member `Some` is inconsistent:

```json5
[
    Option::None
    Option::Some(11)
    Option::Some("John") // The type of this member is not consistent.
]
```

A Variant member can carry a value of any type, such as an Object:

```json5
Option::Some({
    id: 123
    name: "Alice"
})
```

Or a Tuple:

```json5
Option::Some((211, 223))
```

In fact, a Variant member can also carry directly multiple values, which can be either Object-style or Tuple-style, for example:

```json5
// Object-style variant member
Shape::Rectangle{
    width: 307
    height: 311
}
```

and

```json5
// Tuple-style variant member
Color::RGB(255, 127, 63)
```

### 5.3 Comments

Like JavaScript and C/C++, ASON also supports two types of comments: line comments and block comments. Comments are for human readability and are completely ignored by the parser.

Line comments start with the `//` symbol and continue until the end of the line. For example:

```json5
// This is a line comment.
{
    id: 123 // This is also a line comment.
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

Unlike JavaScript and C/C++, ASON block comments support nesting. For example:

```json5
/* 
    This is the first level.
    /*
        This is the second level.
    */
    This is the first level again.
*/  
```

The nesting feature of block comments makes it more convenient for us to comment on a piece of code that **already has a block comment**. If block comments do not support nesting like JavaScript and C/C++, we need to remove the inner block comment first before adding a comment to the outer layer, because the inner block comment symbol `*/` will end the outer block comments, no doubt this is an annoying issue.

### 5.4 Documents

An ASON document can only contain one value (one primitive value or one compound value), like JSON, a typical ASON document is usually an Object or a List. In fact, all types of values are allowed, not limited to Objects or Lists. For example, a Tuple, a Variant, even a number or a string is allowed. Just make sure that a document has exactly one value. For example, the following are both valid ASON documents:

```json5
// Valid ASON document.
(11, "Alice", true)
```

and

```json5
// Valid ASON document.
"Hello World!"
```

While the following two are invalid:

```json5
// Invalid ASON document because there are 2 values.
(11, "Alice", true)
"Hello World!"
```

and

```json5
// Invalid ASON document because there are 3 values.
11, "Alice", true
```

## 6 Mapping between ASON and Rust Data Types

ASON natively supports most Rust data types, including Tuples, Enums and Vectors. Because ASON is also strongly data typed, both serialization and deserialization can ensure data accuracy. In fact, ASON is more compatible with Rust's data types than other data formats (such as JSON, YAML and TOML).

> ASON is a data format that is perfectly compatible with Rust's data types.

The following is a list of supported Rust data types:

- Signed and unsigned integers, from `i8`/`u8` to `i64`/`u64`
- Floating point numbers, including `f32` and `f64`
- Boolean
- Char
- String
- Array, such as `[i32; 4]`
- Vec
- Struct
- HashMap
- Tuple
- Enum

### 6.1 Structs

In general, we use structs in Rust to store a group of related data. Rust structs correspond to ASON `Object`. The following is an example of a struct named "User" and its instance `s1`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String
}

let s1 = User {
    id: 123,
    name: String::from("John")
};
```

The corresponding ASON text for instance `s1` is:

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

let s2 = User {
    id: 123,
    name: String::from("John"),
    address: Box::new(Address{
        city: String::from("Shenzhen"),
        street: String::from("Xinan")
    })
}
```

The corresponding ASON text for instance `s2`:

```json5
{
    id: 123
    name: "John"
    address: {
        city: "Shenzhen"
        street: "Xinan"
    }
}
```

### 6.2 Vecs

`Vec` (vector) is another common data structure in Rust, which is used for storing a series of similar data. `Vec` corresponds to ASON `List`. The following code demonstrates adding a field named `orders` to the struct `User` to store order numbers:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    orders: Vec<i32>
}

let v1 = User {
    id: 123,
    name: String::from("John"),
    orders: vec![11, 13, 17, 19]
};
```

The corresponding ASON text for instance `v1` is:

```json5
{
    id: 123
    name: "John"
    orders: [11, 13, 17, 19]
}
```

The elements in a vector can be either simple data (such as `i32` in the above example) or complex data, such as struct. The following code demonstrates adding a field named `addresses` to the struct `User` to store shipping addresses:

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

let v2 = User {
    id: 123,
    name: String::from("John"),
    address: vec![
        Address {
            city: String::from("Guangzhou"),
            street: String::from("Tianhe")
        },
        Address {
            city: String::from("Shenzhen"),
            street: String::from("Xinan")
        },
    ]
};
```

The corresponding ASON text for instance `v2` is:

```json5
{
    id: 123
    name: "John"
    addresses: [
        {
            city: "Guangzhou"
            street: "Tianhe"
        }
        {
            city: "Shenzhen"
            street: "Xinan"
        }
    ]
}
```

### 6.3 HashMaps

Rust's HashMap corresponds to ASON's Map, e.g. the following creates a HashMap instance `m1` of type `<String, Option<String>>`:

```rust
let mut m1 = HashMap::<String, Option<String>>::new();
m1.insert("foo".to_owned(), Some("hello".to_owned()));
m1.insert("bar".to_owned(), None);
m1.insert("baz".to_owned(), Some("world".to_owned()));
```

The corresponding ASON text for instance `m1` is:

```json5
{
    "foo": Option::Some("hello")
    "bar": Option::None
    "baz": Option::Some("world")
}
```

### 6.4 Tuples

There is another common data type _tuple_ in Rust, which can be considered as structs with omitted field names. Tuple just corresponds to ASON `Tuple`.

For example, in the above example, if you want the order list to include not only the order number but also the order status, you can use the Tuple `(i32, String)` to replace `i32`. The modified code is:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    orders: Vec<(i32, String)>
}

let t1 = User {
    id: 123,
    name: String::from("John"),
    orders: vec![
        (11, String::from("ordered"),
        (13, String::from("shipped"),
        (17, String::from("delivered"),
        (19, String::from("cancelled")
    ]
};
```

The corresponding ASON text for instance `v1` is:

```json5
{
    id: 123
    name: "John"
    orders: [
        (11, "ordered")
        (13, "shipped")
        (17, "delivered")
        (19, "cancelled")
    ]
}
```

It should be noted that in some programming languages, tuples and vectors are not clearly distinguished, but in Rust they are completely different data types. Vectors require that all elements have the same data type (Rust arrays are similar to vectors, but vectors have a variable number of elements, while arrays have a fixed size that cannot be changed after creation), while tuples do not require that their member data types be the same, but do require a fixed number of members. ASON's definition of `Tuple` is consistent with Rust's.

### 6.5 Enums

In the above example, the order status is represented by a string. From historical lessons, we know that a better solution is to use an enum. Rust enum corresponds to ASON `Variant`. The following code uses the enum `Status` to replace the `String` in `Vec<(i32, String)>`.

```rust
#[derive(Serialize, Deserialize)]
enum Status {
    Ordered,
    Shipped,
    Delivered,
    Cancelled
}

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    orders: Vec<(i32, Status)>
}

let e1 = User {
    id: 123,
    name: String::from("John"),
    orders: vec![
        (11, Status::Ordered),
        (13, Status::Shipped),
        (17, Status::Delivered),
        (19, Status::Cancelled)
    ]
};
```

The corresponding ASON text for instance `e1` is:

```json5
{
    id: 123
    name: "John"
    orders: [
        (11, Status::Ordered)
        (13, Status::Shipped)
        (17, Status::Delivered)
        (19, Status::Cancelled)
    ]
}
```

Rust enum type is actually quite powerful, it can not only represent different categories of something but also carry data. For example, consider the following enum `Color`:

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

There are four types of values in Rust enums:

- Without value, e.g., `Color::Transparent`
- With one value, e.g., `Color::Grayscale(u8)`
- Tuple-like with multiple values, e.g., `Color::Rgb(u8, u8, u8)`
- Struct-like with multiple "key-value" pairs, e.g., `Color::Hsl{...}`

ASON `Variant` fully supports all flavours of Rust enums, consider the following instance:

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

The corresponding ASON text for instance `e2` is:

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

The ASON text closely resembles the Rust data literals, which is intentional. The design aims to reduce the learning curve for users by making ASON similar to existing data formats (JSON) and programming languages (Rust).

### 6.6 Other Data Types

Some Rust data types are not supported, includes:

- Octal integer literals
- Unit (i.e. `()`)
- Unit struct, such as `struct Foo;`
- New-type struct, such as `struct Width(u32);`
- Tuple-like struct, such as `struct RGB(u8, u8, u8);`

It is worth nothing that the [serde framework's data model](https://serde.rs/data-model.html) does not include the `DateTime` type, so ASON `DateTime` cannot be directly serialized or deserialized to Rust's `chrono::DateTime`. If you serialize a `chrono::DateTime` type value, you will get a regular string. A workaround is to wrap the `chrono::DateTime` value as an `ason::Date` type. For more details, please refer to the 'test_serialize' unit test in `ason::serde::serde_date::tests` in the library source code.

In addition, serde treats fixed-length arrays such as `[i32; 4]` as tuples rather than vectors, so the Rust array `[11, 13, 17, 19]` will be serialized as ASON Tuple `(11, 13, 17, 19)`.

### 6.7 Default Values

TODO

## 7 ASON Specification

This section describes the ASON specification in detail, it is mainly for developers who want to implement ASON parsers and formatters in other programming languages. For general users, please refer to the previous sections.

TODO

## 8 Linking

- [Source code on GitHub](https://github.com/hemashushu/ason)
- [Crates.io](https://crates.io/crates/ason)

## 9 License

This project is licensed under the MPL 2.0 License with additional terms. See the files [LICENSE](./LICENSE) and [LICENSE.additional](./LICENSE.additional)
