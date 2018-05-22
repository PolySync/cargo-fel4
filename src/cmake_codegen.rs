use super::Error;
use cmake_config::*;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CMakeCodegenError {
    ParseError(ParseError),
    GenerationError(RustCodeGenerationError),
    DuplicateIdentifiers(String),
    WriteIoError,
}

impl From<ParseError> for CMakeCodegenError {
    fn from(p: ParseError) -> Self {
        CMakeCodegenError::ParseError(p)
    }
}

impl From<RustCodeGenerationError> for CMakeCodegenError {
    fn from(r: RustCodeGenerationError) -> Self {
        CMakeCodegenError::GenerationError(r)
    }
}

pub fn filter_to_interesting_flags<I>(i: I) -> Vec<RawFlag>
where
    I: IntoIterator<Item = RawFlag>,
{
    let mut v: Vec<RawFlag> = i.into_iter()
        .filter(|f| f.cmake_type == CMakeType::String || f.cmake_type == CMakeType::Bool)
        .filter(|f| !f.key.starts_with("CMAKE_"))
        .collect();
    v.sort();
    v
}

pub fn cache_to_interesting_flags<P: AsRef<Path>>(
    cmake_cache_path: P,
) -> Result<Vec<RawFlag>, CMakeCodegenError> {
    let all_flags = parse_file_to_raw(cmake_cache_path)?;
    Ok(filter_to_interesting_flags(all_flags))
}

pub fn simple_flags_to_rust_writer<'a, I, W: Write>(
    flags: I,
    writer: &mut W,
    indent_spaces: usize,
) -> Result<(), CMakeCodegenError>
where
    I: IntoIterator<Item = &'a SimpleFlag>,
{
    let mut identifiers: HashSet<String> = HashSet::new();
    for flag in flags {
        let RustConstItem { code, identifier } = flag.generate_rust_const_item()?;
        if identifiers.contains(&identifier) {
            return Err(CMakeCodegenError::DuplicateIdentifiers(identifier));
        } else {
            identifiers.insert(identifier);
        }
        writeln!(writer, "{:indent$}{}", "", code, indent = indent_spaces)
            .map_err(|_| CMakeCodegenError::WriteIoError)?;
    }
    Ok(())
}

pub fn truthy_boolean_flags_as_rust_identifiers<'a, I>(
    flags: I,
) -> Result<Vec<String>, CMakeCodegenError>
where
    I: IntoIterator<Item = &'a SimpleFlag>,
{
    let mut out = Vec::new();
    for active_cmake_bool_prop in flags.into_iter().filter_map(|f| match f {
        SimpleFlag::Stringish(_, _) => None,
        SimpleFlag::Boolish(Key(_), false) => None,
        SimpleFlag::Boolish(Key(k), true) => Some(k),
    }) {
        if !is_valid_rust_identifier(active_cmake_bool_prop) {
            return Err(CMakeCodegenError::GenerationError(
                RustCodeGenerationError::InvalidIdentifier(active_cmake_bool_prop.to_string()),
            ));
        }
        out.push(active_cmake_bool_prop.to_string())
    }

    // sort the result so that we get a deterministic order
    out.sort();

    Ok(out)
}
impl From<CMakeCodegenError> for Error {
    fn from(c: CMakeCodegenError) -> Self {
        match c {
            CMakeCodegenError::ParseError(p) => match p {
                ParseError::IoFailure => {
                    Error::ExitStatusError("Failed to read CMakeCache.txt file".into())
                }
                ParseError::InvalidTypeHint => {
                    Error::ExitStatusError("Invalid type hint in CMakeCache.txt file".into())
                }
                ParseError::PropertyMissingKeyTypeValueTriple => Error::ExitStatusError(
                    "Invalid property definition in CMakeCache.txt file".into(),
                ),
            },
            CMakeCodegenError::GenerationError(r) => match r {
                RustCodeGenerationError::InvalidIdentifier(s) => Error::ExitStatusError(format!(
                    "Invalid identifier interpreted from CMakeCache.txt: {}",
                    s
                )),
                RustCodeGenerationError::InvalidStringLiteral(s) => {
                    Error::ExitStatusError(format!(
                        "Invalid Rust string literal generated from a value in CMakeCache.txt: {}",
                        s
                    ))
                }
            },
            CMakeCodegenError::DuplicateIdentifiers(i) => Error::ExitStatusError(format!(
                "Duplicate identifiers generated in rust config from CMakeCache.txt: {}",
                i
            )),
            CMakeCodegenError::WriteIoError => {
                Error::ExitStatusError("Failure to write out generated rust config.".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO - test invalid cmake flag parsing case
    // TODO - test duplicate identifiers case
    // TODO - test invalid identifiers case
    // TODO - test happy path
    // TODO - test CMAKE_ filtration
    // TODO - test uninteresting CMakeType filtration
    // TODO - test truthy feature flag filtration
    use super::*;
    use std::str;
    #[test]
    fn indentation_control() {
        let f = SimpleFlag::Boolish(Key("A".to_string()), true);
        let mut a: Vec<u8> = Vec::new();
        simple_flags_to_rust_writer(&[f.clone()], &mut a, 0).expect("Oh no");
        assert_eq!("pub const A:bool = true;\n", str::from_utf8(&a).unwrap());

        let mut b: Vec<u8> = Vec::new();
        simple_flags_to_rust_writer(&[f.clone()], &mut b, 4).expect("Oh no");
        assert_eq!(
            "    pub const A:bool = true;\n",
            str::from_utf8(&b).unwrap()
        );
    }
}
