# ASON Specification

ASON is a UTF-8 text format for data serialization, with explicitly typed numbers, Unicode text, and native enumeration support. This document defines its syntax and value types for developers implementing parsers, serializers, formatters, and validation tools.

The specification covers document structure, lexical elements, primitive and compound values, and error reporting. It is independent of any programming language or serialization framework; language-specific APIs and type mappings are outside its scope. The processing pipeline in section 7 describes a typical implementation strategy, not a required internal architecture.

Syntax examples use ASON unless stated otherwise. The EBNF-like grammar in section 5 summarizes the syntax; the accompanying prose defines additional constraints, such as numeric ranges and type consistency. Where the summary and prose differ, the prose takes precedence.

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->

- [1 Document Structure](#1-document-structure)
- [2 Lexical Elements](#2-lexical-elements)
  - [2.1 Encoding](#21-encoding)
  - [2.2 Whitespace](#22-whitespace)
  - [2.3 Commas](#23-commas)
  - [2.4 Comments](#24-comments)
  - [2.5 Punctuation](#25-punctuation)
- [3 Primitive Values](#3-primitive-values)
  - [3.1 Numbers](#31-numbers)
    - [3.1.1 Number Types](#311-number-types)
    - [3.1.2 Decimal Integers](#312-decimal-integers)
    - [3.1.3 Hexadecimal Integers](#313-hexadecimal-integers)
    - [3.1.4 Octal Integers](#314-octal-integers)
    - [3.1.5 Binary Integers](#315-binary-integers)
    - [3.1.6 Decimal Floating-Point Numbers](#316-decimal-floating-point-numbers)
    - [3.1.7 Hexadecimal Floating-Point Numbers](#317-hexadecimal-floating-point-numbers)
    - [3.1.8 Digit Separators](#318-digit-separators)
    - [3.1.9 Type Suffixes](#319-type-suffixes)
    - [3.1.10 Signed Numbers](#3110-signed-numbers)
    - [3.1.11 Special Floating-Point Values](#3111-special-floating-point-values)
  - [3.2 Booleans](#32-booleans)
  - [3.3 Characters](#33-characters)
    - [3.3.1 Escape Sequences](#331-escape-sequences)
  - [3.4 Strings](#34-strings)
    - [3.4.1 Multi-Line Strings](#341-multi-line-strings)
    - [3.4.2 Concatenated Strings](#342-concatenated-strings)
    - [3.4.3 Auto-Trimmed Strings](#343-auto-trimmed-strings)
    - [3.4.4 Raw Strings](#344-raw-strings)
  - [3.5 DateTime](#35-datetime)
  - [3.6 Hexadecimal Byte Data](#36-hexadecimal-byte-data)
- [4 Compound Values](#4-compound-values)
  - [4.1 Lists](#41-lists)
  - [4.2 Maps](#42-maps)
  - [4.3 Tuples](#43-tuples)
  - [4.4 Objects](#44-objects)
  - [4.5 Enumerations](#45-enumerations)
- [5 Grammar Summary](#5-grammar-summary)
- [6 Token Termination](#6-token-termination)
- [7 Processing Pipeline](#7-processing-pipeline)
- [8 Error Handling](#8-error-handling)
- [9 File Extension and MIME Type](#9-file-extension-and-mime-type)
- [10 Links](#10-links)
- [11 License](#11-license)

<!-- /code_chunk_output -->

## 1 Document Structure

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

## 2 Lexical Elements

### 2.1 Encoding

ASON documents are encoded in UTF-8.

### 2.2 Whitespace

The following characters are treated as whitespace and separate tokens. They are ignored by the parser (except within strings and characters):

| Character       | Code Point |
|-----------------|------------|
| Space           | `U+0020`   |
| Tab             | `U+0009`   |
| Carriage Return | `U+000D`   |
| Line Feed       | `U+000A`   |

The CRLF pair (`\r\n`, used on Windows) is treated as a single line break.

### 2.3 Commas

Commas (`,`) serve as optional separators between elements of Lists, Tuples, Objects, and Maps. They carry no semantic meaning and are treated as whitespace by the parser. Trailing commas (after the last element) are permitted.

The following are all equivalent:

```json5
[1,2,3]
[1 2 3]
[1,2,3,]
```

### 2.4 Comments

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

### 2.5 Punctuation

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

Note that `::` (the separator between the type name and the variant name of an Enumeration) is not a standalone token. It is part of an Enumeration token, so no whitespace is allowed around it (see section 4.5).

## 3 Primitive Values

### 3.1 Numbers

ASON supports integers and floating-point numbers. Every number has a specific data type, which is either specified by a type suffix or determined by default.

#### 3.1.1 Number Types

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

#### 3.1.2 Decimal Integers

Decimal integers consist of one or more decimal digits (`0`-`9`).

```text
decimal_integer = "0" | ( non_zero_digit { digit } )
non_zero_digit  = "1" | "2" | ... | "9"
digit           = "0" | "1" | ... | "9"
```

Examples: `0`, `123`, `999_999`

Leading zeros are not permitted for multi-digit decimal integers. `0` alone is valid, but `0123` is invalid.

#### 3.1.3 Hexadecimal Integers

Hexadecimal integers are prefixed with `0x` or `0X`, followed by one or more hexadecimal digits (`0`-`9`, `a`-`f`, `A`-`F`).

```text
hex_integer = "0" ("x" | "X") hex_digit { hex_digit }
hex_digit   = "0"..."9" | "a"..."f" | "A"..."F"
```

Examples: `0xFF`, `0x1A2B`, `0x00`

An empty hexadecimal number (e.g., `0x`) is invalid.

#### 3.1.4 Octal Integers

Octal integers are prefixed with `0o` or `0O`, followed by one or more octal digits (`0`-`7`).

```text
octal_integer = "0" ("o" | "O") octal_digit { octal_digit }
octal_digit   = "0"..."7"
```

Examples: `0o755`, `0o644`

An empty octal number (e.g., `0o`) is invalid. Octal floating-point numbers are not supported.

#### 3.1.5 Binary Integers

Binary integers are prefixed with `0b` or `0B`, followed by one or more binary digits (`0` or `1`).

```text
binary_integer = "0" ("b" | "B") binary_digit { binary_digit }
binary_digit   = "0" | "1"
```

Examples: `0b1010`, `0b1100_0011`

An empty binary number (e.g., `0b`) is invalid. Binary floating-point numbers are not supported.

#### 3.1.6 Decimal Floating-Point Numbers

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

#### 3.1.7 Hexadecimal Floating-Point Numbers

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

#### 3.1.8 Digit Separators

Underscores (`_`) may be inserted between any two characters in a numeric literal to improve readability, and multiple underscores may appear in a row. They are ignored by the parser. However, underscores are not allowed within a prefix (e.g., `0_xFF` is invalid) or in the special values `NaN` and `Inf` (see section 3.1.11).

Examples: `123_456_789`, `0xFF_FF`, `0b1010_0101`, `6.626_070_e-34`

#### 3.1.9 Type Suffixes

An explicit type suffix may be appended to a number to specify its type. The suffix is one of the type names listed in section 3.1.1 (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`).

One or more underscores may appear between the number and its type suffix, but they are optional: `255_u8`, `3.14____f32`, `65u8`.

Rules for type suffixes in different number formats:

- Decimal integers and floating-point numbers: All type suffixes are valid.
- Hexadecimal integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. The `f32` and `f64` suffixes cannot be used because `f` is a valid hexadecimal digit.
- Hexadecimal floating-point numbers (P notation): Only `f32` and `f64` suffixes are valid.
- Octal and binary integers: Only `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` suffixes are valid. Floating-point types are not supported for these formats.

It is an error if a number's value is out of the range of its type.

#### 3.1.10 Signed Numbers

Numbers can be preceded by a `+` (plus) or `-` (minus) sign.

Rules:

- The `+` sign is permitted for signed integer types and floating-point types. It does not change the value. A `+` sign in front of `NaN` is invalid.
- The `-` sign negates the number. It is permitted for signed integer types (`i8`, `i16`, `i32`, `i64`) and floating-point types (`f32`, `f64`). Applying `-` to unsigned types (`u8`, `u16`, `u32`, `u64`) is an error. A `-` sign in front of `NaN` is also invalid.
- Overflow is checked after the sign is applied. For example, `128_i8` is invalid because `128` exceeds `i8::MAX` (127), but `-128_i8` is valid because the range of `i8` includes -128.

#### 3.1.11 Special Floating-Point Values

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

### 3.2 Booleans

Boolean values are represented by the keywords `true` and `false`.

```text
boolean = "true" | "false"
```

### 3.3 Characters

A character literal consists of a single character enclosed in single quotes (`'`).

```text
char = "'" ( character | escape_sequence ) "'"
```

The character can be any valid Unicode scalar value (code points U+0000 to U+D7FF and U+E000 to U+10FFFF).

An empty character (`''`) is invalid.

A character literal containing more than one character (e.g., `'ab'`) is also invalid. Note that some emojis consist of multiple Unicode code points, even though they are displayed as a single character. For example, the emoji 🤦‍♂️ consists of 4 code points: U+1F926, U+200D, U+2642, and U+FE0F. Therefore, this emoji enclosed in single quotes is an invalid character literal.

#### 3.3.1 Escape Sequences

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

### 3.4 Strings

A string literal is a sequence of characters enclosed in double quotes (`"`).

```text
string = '"' { string_char } '"'
string_char = any_character_except_backslash_and_double_quote
            | escape_sequence
```

The same escape sequences as character literals are supported (see section 3.3.1). Double quotes within the string must be escaped as `\"` and backslashes as `\\`.

An empty string (`""`) is valid.

#### 3.4.1 Multi-Line Strings

Strings can span multiple lines. All characters between the opening and closing double quotes (including line breaks, tabs, and spaces) are part of the string content.

```json5
"Line one
    Line two
    Line three"
```

This is equivalent to `"Line one\n    Line two\n    Line three"`. Note that all whitespace characters are preserved as they are.

#### 3.4.2 Concatenated Strings

A backslash (`\`) at the end of a line (immediately before the line break) joins the line with the next one. The line break and all leading whitespace of the next line are removed.

```json5
"The quick brown fox \
    jumps over \
    the lazy dog"
```

This is equivalent to `"The quick brown fox jumps over the lazy dog"`.

The backslash must be immediately followed by `\n` or `\r\n`. The leading whitespace (spaces and tabs) of the continuation line is removed.

#### 3.4.3 Auto-Trimmed Strings

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

#### 3.4.4 Raw Strings

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

### 3.5 DateTime

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

### 3.6 Hexadecimal Byte Data

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

## 4 Compound Values

### 4.1 Lists

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

### 4.2 Maps

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

### 4.3 Tuples

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

### 4.4 Objects

An Object is an ordered sequence of zero or more key-value pairs enclosed in curly braces (`{` and `}`).

```text
object = "{" { key_value_pair [","] } "}"
key_value_pair = identifier ":" value
```

Keys are identifiers (unquoted). They must **not** be enclosed in double quotes (unlike JSON). If a quoted string is used as a key, it is a parse error.

An identifier follows these rules:

- Must start with a letter (`a`-`z`, `A`-`Z`), an underscore (`_`), or a Unicode character in the range U+00A0 to U+D7FF or U+E000 to U+10FFFF (which includes CJK characters, Greek letters, emoji, etc.).
- May continue with letters, digits (`0`-`9`), underscores, or Unicode characters in the same ranges.
- Must not contain `::`. An identifier containing `::` becomes an Enumeration token (see section 4.5).

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

### 4.5 Enumerations

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

## 5 Grammar Summary

The following is a summary of the ASON grammar in EBNF-like notation. The rules for tokens (numbers, strings, identifiers, etc.) describe exact character sequences. Whitespace, commas, and comments can appear between any two tokens and are discarded by the lexer (see section 2), so they are omitted from the rules for compound values.

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

## 6 Token Termination

The following characters (called terminators) end a token:

- Whitespace: space (` `), tab (`\t`), carriage return (`\r`), line feed (`\n`)
- Comma: `,`
- Punctuation: `:`, `{`, `}`, `[`, `]`, `(`, `)`
- Comment start: `/` (when followed by `/` or `*`)
- End of file (`EOF`)

This means that tokens such as numbers, identifiers, and keywords do not need explicit delimiters; they end when one of the above characters is encountered.

## 7 Processing Pipeline

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

## 8 Error Handling

ASON errors fall into three categories:

- Errors with a position: Point to a specific character in the source (index, line number, and column number). These typically occur during lexing when an unexpected character is encountered.
- Errors with a range: Point to a span of characters in the source (start position and end position). These typically occur when a token or structure contains an error (e.g., number overflow or an invalid escape sequence).
- Unexpected end of document: Indicates that the input ended prematurely (e.g., an unclosed string or a missing closing bracket).

## 9 File Extension and MIME Type

The file extension for ASON documents is `.ason`, and the MIME type is `application/ason`.

## 10 Links

- [ASON project and Rust reference implementation](https://github.com/hemashushu/ason)
- [Issue tracker](https://github.com/hemashushu/ason/issues) for specification questions, clarifications, and proposed changes
- [ASONB (ASON Binary)](https://github.com/hemashushu/asonb), the binary counterpart of ASON
- [The Unicode Standard](https://www.unicode.org/versions/latest/) for Unicode scalar values and UTF-8 encoding
- [RFC 3339: Date and Time on the Internet](https://datatracker.ietf.org/doc/html/rfc3339), the basis for the DateTime syntax with the relaxations specified in section 3.5
- [IEEE 754 floating-point arithmetic](https://en.wikipedia.org/wiki/IEEE_754), the basis for the `f32` and `f64` numeric types

## 11 License

This document is distributed as part of the ASON project, which is licensed under the Mozilla Public License 2.0 (MPL-2.0) with additional terms. See [LICENSE](../LICENSE) and [LICENSE.additional](../LICENSE.additional) for the full terms.
