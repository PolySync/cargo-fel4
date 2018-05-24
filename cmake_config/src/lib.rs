/*!

This crate provides a parser for CMakeCache.txt files.

The primary entry points are `parse_raw` and `parse_file_to_raw`, which produces
a vector of the minimally-processed CMake flags from a supplied
path.

```
use cmake_config::{CMakeType, RawFlag};
use std::io::{BufReader, Cursor};
let s = r"FOO:BOOL=ON
BAR:STRING=BAZ";
let reader = BufReader::new(Cursor::new(s));
let flags = cmake_config::parse_raw(reader).expect("Parsing error!");
assert_eq!(vec![
            RawFlag {
                key: "FOO".into(),
                cmake_type: CMakeType::Bool,
                value: "ON".into(),
            },
            RawFlag {
                key: "BAR".into(),
                cmake_type: CMakeType::String,
                value: "BAZ".into(),
            },
], flags);
```

Optionally, Flags can be summarized from their initial `RawFlag` form
into `SimpleFlag` instances for representation in Rust.

```
use cmake_config::{CMakeType, RawFlag, SimpleFlag, Key};
let raw = RawFlag {
          key: "FOO".into(),
          cmake_type: CMakeType::Bool,
          value: "ON".into(),
};
let simplified = SimpleFlag::from(&raw);
assert_eq!(SimpleFlag::Boolish(Key("FOO".into()), true), simplified);
```

`SimpleFlag` also provides a convenience method, `generate_rust_const_item`,
which produces the text of a Rust-lang const definition for that flag.

*/
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate syn;

#[cfg(test)]
#[macro_use]
extern crate proptest;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::Path;
use syn::{parse_str, Expr, ExprLit, Lit};

lazy_static! {
    static ref RUST_VALID_IDENTIFIER_REGEX: Regex =
        Regex::new("(^[a-zA-Z][a-zA-Z0-9_]*$)|(^_[a-zA-Z0-9_]+$)").unwrap();
}

/// Represents a single CMake property
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RawFlag {
    pub key: String,
    pub cmake_type: CMakeType,
    pub value: String,
}

/// The type hint associated with a CMake property
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum CMakeType {
    Bool,
    Path,
    FilePath,
    String,
    Internal,
    Static,
    Uninitialized,
}

/// A pared-down and interpreted representation
/// of a CMake flag
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SimpleFlag {
    Stringish(Key, String),
    Boolish(Key, bool),
}

/// A newtype wrapper for the key / property-name
/// of a CMake flag. Mostly here to avoid confusion
/// between the similarly-shaped key and value of a
/// `SimpleFlag::Stringish` variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Key(pub String);

impl<S> From<S> for Key
where
    S: AsRef<str>,
{
    fn from(s: S) -> Self {
        Key(s.as_ref().to_string())
    }
}

impl<'a> From<&'a RawFlag> for SimpleFlag {
    fn from(raw: &'a RawFlag) -> Self {
        match raw.cmake_type {
            CMakeType::Bool => SimpleFlag::Boolish(
                Key(raw.key.clone()),
                interpret_value_as_boolish(raw.value.clone()),
            ),
            CMakeType::Path
            | CMakeType::FilePath
            | CMakeType::String
            | CMakeType::Internal
            | CMakeType::Static
            | CMakeType::Uninitialized => {
                SimpleFlag::Stringish(Key(raw.key.clone()), raw.value.clone())
            }
        }
    }
}

impl SimpleFlag {
    /// Produce code that could be used in a Rust language file
    /// to represent the flag as a `const` item.
    pub fn generate_rust_const_item(&self) -> Result<RustConstItem, RustCodeGenerationError> {
        match self {
            SimpleFlag::Stringish(Key(k), v) => {
                if !is_valid_rust_identifier(&k) {
                    return Err(RustCodeGenerationError::InvalidIdentifier(k.clone()));
                }
                let literal = format!("\"{}\"", &v);
                let valid_literal = match parse_str::<Expr>(&literal) {
                    Ok(Expr::Lit(ExprLit {
                        lit: Lit::Str(_),
                        ..
                    })) => true,
                    _ => false,
                };
                // TODO - consider applying string escaping as a fallback?
                if !valid_literal {
                    return Err(RustCodeGenerationError::InvalidStringLiteral(v.clone()));
                }
                let code = format!("pub const {}:&'static str = {};", &k, &literal);
                Ok(RustConstItem {
                    code,
                    identifier: k.clone(),
                })
            }
            SimpleFlag::Boolish(Key(k), v) => {
                if !is_valid_rust_identifier(&k) {
                    return Err(RustCodeGenerationError::InvalidIdentifier(k.clone()));
                }
                Ok(RustConstItem {
                    code: format!("pub const {}:bool = {};", &k, v),
                    identifier: k.clone(),
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct RustConstItem {
    pub code: String,
    pub identifier: String,
}

/// Convenience function determining if a string is usable without alteration
/// as a Rust identifier for an item.
pub fn is_valid_rust_identifier<S: AsRef<str>>(s: S) -> bool {
    RUST_VALID_IDENTIFIER_REGEX.is_match(s.as_ref())
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RustCodeGenerationError {
    InvalidIdentifier(String),
    InvalidStringLiteral(String),
}

pub fn interpret_value_as_boolish<S: AsRef<str>>(s: S) -> bool {
    match s.as_ref().trim().to_uppercase().as_ref() {
        "OFF" | "FALSE" | "NO" | "N" | "NOTFOUND" | "0" => false,
        r if r.is_empty() || r.ends_with("-NOTFOUND") => false,
        _ => true,
    }
}

impl CMakeType {
    pub fn parse<T: AsRef<str>>(s: T) -> Option<CMakeType> {
        match s.as_ref() {
            "BOOL" => Some(CMakeType::Bool),
            "PATH" => Some(CMakeType::Path),
            "FILEPATH" => Some(CMakeType::FilePath),
            "STRING" => Some(CMakeType::String),
            "INTERNAL" => Some(CMakeType::Internal),
            "STATIC" => Some(CMakeType::Static),
            "UNINITIALIZED" => Some(CMakeType::Uninitialized),
            _ => None,
        }
    }

    pub fn cmake_name(&self) -> &'static str {
        match *self {
            CMakeType::Bool => "BOOL",
            CMakeType::Path => "PATH",
            CMakeType::FilePath => "FILEPATH",
            CMakeType::String => "STRING",
            CMakeType::Internal => "INTERNAL",
            CMakeType::Static => "STATIC",
            CMakeType::Uninitialized => "UNINITIALIZED",
        }
    }
}

/// The usual things that might go wrong when interpreting
/// a CMakeCache blob of data
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidTypeHint,
    PropertyMissingKeyTypeValueTriple,
    IoFailure,
}

impl From<IoError> for ParseError {
    fn from(_: IoError) -> Self {
        ParseError::IoFailure
    }
}

/// Most generic entry point, intended to interpret a buffered reader
/// of CMakeCache textual data into the represented flags.
///
/// Technically could be formulate-able as returning an iterator, and presently
/// only uses `Vec` out of convenience.
pub fn parse_raw<B: BufRead>(b: B) -> Result<Vec<RawFlag>, ParseError> {
    let mut v = Vec::new();
    for line_result in b.lines() {
        let line = line_result?;
        if let Some(flag) = parse_line(line)? {
            v.push(flag);
        }
    }
    Ok(v)
}

/// Convenience wrapper around `parse_raw` for reading a CMakeCache.txt file
/// and handing back either a `Vec` of flags or a very simple error.
pub fn parse_file_to_raw<P: AsRef<Path>>(file_path: P) -> Result<Vec<RawFlag>, ParseError> {
    parse_raw(BufReader::new(File::open(file_path)?))
}

fn parse_line<S: AsRef<str>>(l: S) -> Result<Option<RawFlag>, ParseError> {
    let line = l.as_ref().trim();
    // skip comments and empty lines
    if line.starts_with('#') || line.starts_with("//") || line.is_empty() {
        return Ok(None);
    }
    // split line into tokens: key:type=value -> ["key", "type", "value"]
    let tokens: Vec<&str> = line.split(|c| c == ':' || c == '=').collect();
    if tokens.len() < 3 {
        return Err(ParseError::PropertyMissingKeyTypeValueTriple);
    }
    let (key, maybe_type_hint, value) = (
        tokens[0].trim(),
        CMakeType::parse(tokens[1].trim()),
        tokens[2].trim(),
    );
    let type_hint = if let Some(th) = maybe_type_hint {
        th
    } else {
        return Err(ParseError::InvalidTypeHint);
    };
    let flag = RawFlag {
        key: key.to_string(),
        cmake_type: type_hint,
        value: value.to_string(),
    };
    Ok(Some(flag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parse_line_empty_edge_cases() {
        assert_eq!(Ok(None), parse_line(""));
        assert_eq!(Ok(None), parse_line("# Comment"));
        assert_eq!(Ok(None), parse_line("// Comment"));
        assert_eq!(Ok(None), parse_line("#key:BOOL=TRUE"));
        assert_eq!(Ok(None), parse_line("//key:BOOL=TRUE"));
    }

    #[test]
    fn parse_line_missing_content() {
        assert_eq!(
            Err(ParseError::PropertyMissingKeyTypeValueTriple),
            parse_line("=")
        );
        assert_eq!(
            Err(ParseError::PropertyMissingKeyTypeValueTriple),
            parse_line(":")
        );
        assert_eq!(Err(ParseError::InvalidTypeHint), parse_line(":="));
        assert_eq!(Err(ParseError::InvalidTypeHint), parse_line("=:"));
    }

    #[test]
    fn sanity_check_bool_const_ok() {
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const HELLO_WORLD:bool = false;".to_string(),
                identifier: "HELLO_WORLD".to_string(),
            }),
            SimpleFlag::Boolish(Key("HELLO_WORLD".to_string()), false).generate_rust_const_item()
        );
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const helloWorld:bool = false;".to_string(),
                identifier: "helloWorld".to_string(),
            }),
            SimpleFlag::Boolish(Key("helloWorld".to_string()), false).generate_rust_const_item()
        );
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const hello_world:bool = false;".to_string(),
                identifier: "hello_world".to_string(),
            }),
            SimpleFlag::Boolish(Key("hello_world".to_string()), false).generate_rust_const_item()
        );
    }

    #[test]
    fn sanity_check_bool_const_error() {
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("_".into())),
            SimpleFlag::Boolish(Key("_".to_string()), false).generate_rust_const_item()
        );
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("0".into())),
            SimpleFlag::Boolish(Key("0".to_string()), false).generate_rust_const_item()
        );
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("0ABC".into())),
            SimpleFlag::Boolish(Key("0ABC".to_string()), false).generate_rust_const_item()
        );
    }

    #[test]
    fn sanity_check_str_const() {
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const HELLO_WORLD:&'static str = \"whatever\";".to_string(),
                identifier: "HELLO_WORLD".to_string(),
            }),
            SimpleFlag::Stringish(Key("HELLO_WORLD".to_string()), "whatever".to_string())
                .generate_rust_const_item()
        );
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const helloWorld:&'static str = \"whatever\";".to_string(),
                identifier: "helloWorld".to_string(),
            }),
            SimpleFlag::Stringish(Key("helloWorld".to_string()), "whatever".to_string())
                .generate_rust_const_item()
        );
        assert_eq!(
            Ok(RustConstItem {
                code: "pub const hello_world:&'static str = \"whatever\";".to_string(),
                identifier: "hello_world".to_string(),
            }),
            SimpleFlag::Stringish(Key("hello_world".to_string()), "whatever".to_string())
                .generate_rust_const_item()
        );
    }

    #[test]
    fn sanity_check_str_const_id_error() {
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("_".into())),
            SimpleFlag::Stringish(Key("_".to_string()), "a".into()).generate_rust_const_item()
        );
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("0".into())),
            SimpleFlag::Stringish(Key("0".to_string()), "a".into()).generate_rust_const_item()
        );
        assert_eq!(
            Err(RustCodeGenerationError::InvalidIdentifier("0ABC".into())),
            SimpleFlag::Stringish(Key("0ABC".to_string()), "a".into()).generate_rust_const_item()
        );
    }

    #[test]
    fn sanity_check_str_const_literal_error() {
        assert_eq!(
            Err(RustCodeGenerationError::InvalidStringLiteral("\\".into())),
            SimpleFlag::Stringish(Key("a".to_string()), "\\".into()).generate_rust_const_item()
        );
        assert_eq!(
            Err(RustCodeGenerationError::InvalidStringLiteral("\"".into())),
            SimpleFlag::Stringish(Key("a".to_string()), "\"".into()).generate_rust_const_item()
        );
    }

    fn arb_cmake_type() -> BoxedStrategy<CMakeType> {
        prop_oneof![
            Just(CMakeType::Bool),
            Just(CMakeType::Path),
            Just(CMakeType::FilePath),
            Just(CMakeType::String),
            Just(CMakeType::Internal),
            Just(CMakeType::Static),
            Just(CMakeType::Uninitialized),
        ].boxed()
    }

    prop_compose! {
        fn arb_valid_rustificable_key()(ref key in r"([a-zA-Z][a-zA-Z0-9_]+)|(_[a-zA-Z][a-zA-Z0-9_]+)") -> String {
            key.to_string()
        }
    }

    prop_compose! {
        fn arb_raw_flag()(ref key in arb_valid_rustificable_key(),
                          ref t in arb_cmake_type(),
                          ref val in "[^\\s:=#/\"\\\\]*") -> RawFlag {
            RawFlag {
               key: key.to_string(),
               cmake_type: t.clone(),
               value: val.to_string(),
            }
        }
    }

    prop_compose! {
        fn arb_simple_flag()(ref raw_flag in arb_raw_flag()) -> SimpleFlag {
            SimpleFlag::from(raw_flag)
        }
    }

    proptest! {
        #[test]
        fn arbitrary_string_no_panic(ref l in ".*") {
            let _ = parse_line(l);
        }


        #[test]
        fn arbitrary_valid_parseable_raw(ref raw_flag in arb_raw_flag()) {
            let expected = raw_flag.clone();
            let l = format!("{}:{}={}", raw_flag.key, raw_flag.cmake_type.cmake_name(), raw_flag.value);
            let f = parse_line(l).expect("Should be parseable!");
            assert_eq!(Some(expected), f);
        }

        #[test]
        fn arbitrary_valid_raw_refinable(ref raw_flag in arb_raw_flag()) {
            let f:SimpleFlag = SimpleFlag::from(raw_flag);
            if raw_flag.cmake_type == CMakeType::Bool {
                if let SimpleFlag::Boolish(k, _) = f {
                    assert_eq!(Key(raw_flag.key.clone()), k);
                } else {
                    panic!("Should have been Boolish");
                }
            } else {
                if let SimpleFlag::Stringish(k, v) = f {
                    assert_eq!(Key(raw_flag.key.clone()), k);
                    assert_eq!(raw_flag.value, v);
                } else {
                    panic!("Should have been Stringish");
                }
            }
        }

        #[test]
        fn arbitrary_simple_flag_const_able(ref simple_flag in arb_simple_flag()) {
            let _ = simple_flag.generate_rust_const_item().expect("Should be able to const-ify anything with a rust-compatible identifier");
        }

        #[test]
        fn round_trip_cmake_type(ref t in arb_cmake_type()) {
            let expected = t.clone();
            assert_eq!(Some(expected), CMakeType::parse(t.cmake_name()))
        }
    }
}
