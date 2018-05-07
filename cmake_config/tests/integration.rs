extern crate cmake_config;

use cmake_config::{CMakeType, Key, RawFlag, SimpleFlag};
use std::io::{BufReader, Cursor};
use std::path::*;

#[test]
fn sanity_check_direct_from_string() {
    let s = r"FOO:BOOL=ON
    BAR:STRING=BAZ";
    let reader = BufReader::new(Cursor::new(s));

    let v: Vec<SimpleFlag> = cmake_config::parse_raw(reader)
        .expect("Could not parse content")
        .into_iter()
        .map(|raw_flag| SimpleFlag::from(raw_flag))
        .collect();

    assert_eq!(
        vec![
            SimpleFlag::Boolish(Key("FOO".to_string()), true),
            SimpleFlag::Stringish(Key("BAR".to_string()), "BAZ".to_string()),
        ],
        v
    );
}

#[test]
fn sanity_check_tiny_file_parsing() {
    let v = cmake_config::parse_file_to_raw(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/CMakeCache_tiny.txt"),
    ).expect("could not parse file");
    assert_eq!(
        vec![
            RawFlag {
                key: "FP_PROP".into(),
                cmake_type: CMakeType::FilePath,
                value: "/home/whoever/wherever".into(),
            },
            RawFlag {
                key: "S_PROP_SOME".into(),
                cmake_type: CMakeType::String,
                value: "foo".into(),
            },
            RawFlag {
                key: "S_PROP_NONE".into(),
                cmake_type: CMakeType::String,
                value: "".into(),
            },
            RawFlag {
                key: "B_PROP_ON".into(),
                cmake_type: CMakeType::Bool,
                value: "ON".into(),
            },
            RawFlag {
                key: "B_PROP_OFF".into(),
                cmake_type: CMakeType::Bool,
                value: "OFF".into(),
            },
            RawFlag {
                key: "B_PROP_TRUE".into(),
                cmake_type: CMakeType::Bool,
                value: "TRUE".into(),
            },
            RawFlag {
                key: "B_PROP_FALSE".into(),
                cmake_type: CMakeType::Bool,
                value: "FALSE".into(),
            },
        ],
        v
    );

    let simplified: Vec<SimpleFlag> = v.into_iter()
        .map(|raw_flag| SimpleFlag::from(raw_flag))
        .collect();

    assert_eq!(
        vec![
            SimpleFlag::Stringish("FP_PROP".into(), "/home/whoever/wherever".into()),
            SimpleFlag::Stringish("S_PROP_SOME".into(), "foo".into()),
            SimpleFlag::Stringish("S_PROP_NONE".into(), "".into()),
            SimpleFlag::Boolish("B_PROP_ON".into(), true),
            SimpleFlag::Boolish("B_PROP_OFF".into(), false),
            SimpleFlag::Boolish("B_PROP_TRUE".into(), true),
            SimpleFlag::Boolish("B_PROP_FALSE".into(), false),
        ],
        simplified
    );
}
