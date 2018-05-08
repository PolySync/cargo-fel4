use cmake_config::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub enum CMakeCodegenError {
    ReadIoError,
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
    i.into_iter()
        .filter(|f| f.cmake_type == CMakeType::String || f.cmake_type == CMakeType::Bool)
        .filter(|f| !f.key.starts_with("CMAKE_"))
        .collect()
}
pub fn cache_to_interesting_flags<P: AsRef<Path>>(
    cmake_cache_path: P
) -> Result<Vec<RawFlag>, CMakeCodegenError> {
    let all_flags = parse_file_to_raw(cmake_cache_path)?;
    Ok(filter_to_interesting_flags(all_flags))
}

pub fn cache_to_rust_file<P: AsRef<Path>>(
    cmake_cache_path: P,
    output_rust_file_path: P,
) -> Result<(), CMakeCodegenError> {
    flags_to_rust_file(cache_to_interesting_flags(cmake_cache_path)?, output_rust_file_path)
}

pub fn flags_to_rust_file<I, P: AsRef<Path>>(
    flags: I,
    output_rust_file_path: P,
) -> Result<(), CMakeCodegenError>
where
    I: IntoIterator<Item = RawFlag>,
{
    let out_file =
        File::create(&output_rust_file_path).map_err(|_| CMakeCodegenError::WriteIoError)?;
    let writer = BufWriter::new(out_file);
    flags_to_rust_writer(flags, writer)
}

pub fn flags_to_rust_writer<I, W: Write>(flags: I, mut writer: W) -> Result<(), CMakeCodegenError>
where
    I: IntoIterator<Item = RawFlag>,
{
    let mut identifiers: HashSet<String> = HashSet::new();
    for flag in flags {
        let simplified = SimpleFlag::from(flag);
        let RustConstItem { code, identifier } = simplified.generate_rust_const_item()?;
        if identifiers.contains(&identifier) {
            return Err(CMakeCodegenError::DuplicateIdentifiers(identifier));
        } else {
            identifiers.insert(identifier);
        }
        writeln!(writer, "{}", code).map_err(|_| CMakeCodegenError::WriteIoError)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO - test invalid cmake flag parsing case
    // TODO - test duplicate identifiers case
    // TODO - test invalid identifiers case
    // TODO - test happy path
    // TODO - test CMAKE_ filtration
    // TODO - test uninteresting CMakeType filtration
}
