#![cfg(test)]

mod entry;

use std::fs::DirBuilder;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::time::Duration;

use tempfile::Builder;
use tempfile::TempDir;
use rust_decimal::Decimal;
use rand::seq::SliceRandom;

use maplit::btreemap;
use rust_decimal_macros::dec;

use crate::metadata::schema::SchemaFormat;
use crate::metadata::target::Target;
use crate::metadata::value::Value;
use crate::metadata::value::Sequence;
use crate::metadata::value::Mapping;
use crate::metadata::block::Block;
use crate::metadata::schema::Schema;

use self::entry::DEFAULT_FLAGGER;
use self::entry::DEFAULT_LIBRARY;

trait TestSerialize {
    const INDENT: &'static str = "  ";
    const YAML_LIST_ITEM: &'static str = "- ";

    fn indent_chunk(s: String) -> String {
        let mut to_join = vec![];

        for line in s.lines() {
            to_join.push(format!("{}{}", Self::INDENT, line));
        }

        to_join.join("\n")
    }

    fn indent_yaml_list_chunk(s: String) -> String {
        let mut to_join = vec![];

        for (i, line) in s.lines().enumerate() {
            let prefix = if i == 0 { Self::YAML_LIST_ITEM } else { Self::INDENT };

            to_join.push(format!("{}{}", prefix, line));
        }

        to_join.join("\n")
    }

    fn to_serialized_chunk(&self, schema_format: SchemaFormat) -> String;
}

impl TestSerialize for Schema {
    fn to_serialized_chunk(&self, schema_format: SchemaFormat) -> String {
        match self {
            &Schema::One(ref mb) => Value::Mapping(mb.clone()).to_serialized_chunk(schema_format),
            &Schema::Seq(ref mb_seq) => {
                Value::Sequence(
                    mb_seq
                        .into_iter()
                        .map(|v| Value::Mapping(v.clone()))
                        .collect()
                ).to_serialized_chunk(schema_format)
            },
            &Schema::Map(ref mb_map) => {
                Value::Mapping(
                    mb_map
                        .into_iter()
                        .map(|(k, v)| (k.clone(), Value::Mapping(v.clone())))
                        .collect()
                ).to_serialized_chunk(schema_format)
            },
        }
    }
}

impl TestSerialize for Value {
    fn to_serialized_chunk(&self, schema_format: SchemaFormat) -> String {
        match (schema_format, self) {
            (SchemaFormat::Json, &Self::Null) => "null".into(),
            (SchemaFormat::Yaml, &Self::Null) => "~".into(),
            (SchemaFormat::Json, &Self::String(ref s)) => format!(r#""{}""#, s),
            (SchemaFormat::Yaml, &Self::String(ref s)) => s.clone(),
            (_, &Self::Integer(i)) => format!("{}", i),
            (_, &Self::Decimal(ref d)) => format!("{}", d),
            (_, &Self::Boolean(b)) => format!("{}", b),
            (SchemaFormat::Json, &Self::Sequence(ref seq)) => {
                let mut val_chunks = vec![];

                for val in seq {
                    let val_chunk = val.to_serialized_chunk(schema_format);

                    let val_chunk = Self::indent_chunk(val_chunk);

                    val_chunks.push(val_chunk);
                }

                if val_chunks.len() > 0 {
                    format!("[\n{}\n]", val_chunks.join(",\n"))
                }
                else {
                    String::from("[]")
                }
            },
            (SchemaFormat::Yaml, &Self::Sequence(ref seq)) => {
                let mut val_chunks = vec![];

                for val in seq {
                    let val_chunk = val.to_serialized_chunk(schema_format);

                    let val_chunk = Self::indent_yaml_list_chunk(val_chunk);

                    val_chunks.push(val_chunk);
                }

                if val_chunks.len() > 0 {
                    format!("{}", val_chunks.join("\n"))
                }
                else {
                    String::from("[]")
                }
            },
            (SchemaFormat::Json, &Self::Mapping(ref map)) => {
                let mut kv_pair_chunks = vec![];

                for (key, val) in map {
                    let val_chunk = val.to_serialized_chunk(schema_format);

                    let kv_pair_chunk = format!(r#""{}": {}"#, key, val_chunk);

                    let kv_pair_chunk = Self::indent_chunk(kv_pair_chunk);

                    kv_pair_chunks.push(kv_pair_chunk);
                }

                if kv_pair_chunks.len() > 0 {
                    format!("{{\n{}\n}}", kv_pair_chunks.join(",\n"))
                }
                else {
                    String::from("{}")
                }
            },
            (SchemaFormat::Yaml, &Self::Mapping(ref map)) => {
                let mut kv_pair_chunks = vec![];

                for (key, val) in map {
                    let val_chunk = {
                        let val_chunk = val.to_serialized_chunk(schema_format);

                        match val {
                            Self::Sequence(..) | Self::Mapping(..) => format!("\n{}", Self::indent_chunk(val_chunk)),
                            _ => format!(" {}", val_chunk),
                        }
                    };

                    let kv_pair_chunk = format!("{}:{}", key, val_chunk);

                    kv_pair_chunks.push(kv_pair_chunk);
                }

                if kv_pair_chunks.len() > 0 {
                    format!("{}", kv_pair_chunks.join("\n"))
                }
                else {
                    String::from("{}")
                }
            },
        }
    }
}

pub(crate) struct TestUtil;

impl TestUtil {
    pub const STRING_KEY: &'static str = "string_key";
    pub const INTEGER_KEY: &'static str = "integer_key";
    pub const DECIMAL_KEY: &'static str = "decimal_key";
    pub const BOOLEAN_KEY: &'static str = "boolean_key";
    pub const NULL_KEY: &'static str = "null_key";
    pub const SEQUENCE_KEY: &'static str = "sequence_key";
    pub const MAPPING_KEY: &'static str = "mapping_key";

    pub fn create_temp_media_test_dir(name: &str) -> TempDir {
        let temp_dir = Builder::new().suffix(name).tempdir().unwrap();

        DEFAULT_LIBRARY.realize(&temp_dir.path(), &DEFAULT_FLAGGER);

        std::thread::sleep(Duration::from_millis(1));
        temp_dir
    }

    pub fn sample_string() -> Value {
        Value::String(String::from("string"))
    }

    pub fn sample_integer() -> Value {
        Value::Integer(27)
    }

    pub fn sample_decimal() -> Value {
        Value::Decimal(dec!(3.1415))
    }

    pub fn sample_boolean() -> Value {
        Value::Boolean(true)
    }

    pub fn sample_null() -> Value {
        Value::Null
    }

    pub fn core_flat_sequence() -> Sequence {
        vec![
            Self::sample_string(),
            Self::sample_integer(),
            Self::sample_decimal(),
            Self::sample_boolean(),
            Self::sample_null(),
        ]
    }

    pub fn core_flat_mapping() -> Mapping {
        btreemap![
            String::from(Self::STRING_KEY) => Self::sample_string(),
            String::from(Self::INTEGER_KEY) => Self::sample_integer(),
            String::from(Self::DECIMAL_KEY) => Self::sample_decimal(),
            String::from(Self::BOOLEAN_KEY) => Self::sample_boolean(),
            String::from(Self::NULL_KEY) => Self::sample_null(),
        ]
    }

    pub fn core_nested_mapping() -> Mapping {
        let mut map = Self::core_flat_mapping();

        map.insert(String::from(Self::SEQUENCE_KEY), Self::sample_flat_sequence());
        map.insert(String::from(Self::MAPPING_KEY), Self::sample_flat_mapping());

        map
    }

    pub fn sample_flat_sequence() -> Value {
        Value::Sequence(Self::core_flat_sequence())
    }

    pub fn sample_flat_mapping() -> Value {
        Value::Mapping(Self::core_flat_mapping())
    }

    pub fn core_number_sequence(int_max: i64, dec_extremes: bool, shuffle: bool, include_zero: bool) -> Sequence {
        let mut nums = vec![];

        for i in 1..=int_max {
            nums.push(Value::Integer(i));
            nums.push(Value::Integer(-i));

            // Add -0.5 decimal values.
            let m = (i - 1) * 10 + 5;
            nums.push(Value::Decimal(Decimal::new(m.into(), 1)));
            nums.push(Value::Decimal(Decimal::new((-m).into(), 1)));
        }

        if dec_extremes {
            // These are +/-(int_max + 0.5).
            let m = int_max * 10 + 5;
            nums.push(Value::Decimal(Decimal::new(m.into(), 1)));
            nums.push(Value::Decimal(Decimal::new((-m).into(), 1)));
        }

        if include_zero {
            nums.push(Value::Integer(0));
        }

        if shuffle {
            nums.shuffle(&mut rand::thread_rng());
        }

        nums
    }

    pub fn sample_number_sequence(int_max: i64, dec_extremes: bool, shuffle: bool, include_zero: bool) -> Value {
        Value::Sequence(Self::core_number_sequence(int_max, dec_extremes, shuffle, include_zero))
    }

    // pub fn sample_nested_sequence() -> Value {
    //     Value::Sequence(Self::core_nested_sequence())
    // }

    // pub fn sample_nested_mapping() -> Value {
    //     Value::Mapping(Self::core_nested_mapping())
    // }

    pub fn sample_meta_block(meta_target: Target, target_name: &str, include_flag_key: bool) -> Block {
        let mut map = Self::core_nested_mapping();

        map.insert(
            String::from(format!("{}_key", meta_target.default_file_name())),
            Value::String(format!("{}_val", meta_target.default_file_name())),
        );

        map.insert(
            String::from("meta_target"),
            Value::String(String::from(meta_target.default_file_name())),
        );

        map.insert(
            String::from("target_file_name"),
            Value::String(String::from(target_name)),
        );

        if include_flag_key {
            map.insert(
                String::from("flag_key"),
                Value::String(String::from(target_name)),
            );
        }

        map
    }

    // /// Used for test scenarios where a target is not needed.
    // pub fn sample_naive_meta_block(target_name: &str, include_flag_key: bool) -> Block {
    //     let mut map = Self::core_nested_mapping();

    //     map.insert(
    //         String::from("target_file_name"),
    //         Value::String(String::from(target_name)),
    //     );

    //     if include_flag_key {
    //         map.insert(
    //             String::from("flag_key"),
    //             Value::String(String::from(target_name)),
    //         );
    //     }

    //     map
    // }

    // pub fn create_fixed_value_stream<'a, II>(mvs: II) -> FixedValueStream<'a>
    // where
    //     II: IntoIterator<Item = Value<'a>>,
    // {
    //     FixedValueStream::new(mvs.into_iter().map(|mv| (Cow::Borrowed(Path::new("dummy")), mv)))
    // }

    pub fn create_plain_fanout_test_dir(name: &str, fanout: usize, max_depth: usize) -> TempDir {
        let root_dir = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");

        fn fill_dir(p: &Path, db: &DirBuilder, fanout: usize, breadcrumbs: Vec<usize>, max_depth: usize) {
            for i in 0..fanout {
                let mut new_breadcrumbs = breadcrumbs.clone();

                new_breadcrumbs.push(i);

                let name = if new_breadcrumbs.len() == 0 {
                    String::from("ROOT")
                }
                else {
                    new_breadcrumbs.iter().map(|n| format!("{}", n)).collect::<Vec<_>>().join("_")
                };

                let new_path = p.join(&name);

                if breadcrumbs.len() >= max_depth {
                    // Create files.
                    File::create(&new_path).expect("unable to create file");
                }
                else {
                    // Create dirs and then recurse.
                    db.create(&new_path).expect("unable to create directory");
                    fill_dir(&new_path, &db, fanout, new_breadcrumbs, max_depth);
                }
            }
        }

        let db = DirBuilder::new();

        fill_dir(root_dir.path(), &db, fanout, vec![], max_depth);

        root_dir
    }

    pub fn flag_set_by_default(depth_left: usize, fanout_index: usize) -> bool {
        ((depth_left % 2 == 1) ^ (fanout_index % 2 == 1)) && depth_left <= 1
    }

    pub fn flag_set_by_all(_: usize, _: usize) -> bool {
        true
    }

    pub fn flag_set_by_none(_: usize, _: usize) -> bool {
        false
    }

    pub fn create_meta_fanout_test_dir(name: &str, fanout: usize, max_depth: usize, flag_set_by: fn(usize, usize) -> bool) -> TempDir
    {
        let root_dir = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");

        fn fill_dir(p: &Path, db: &DirBuilder, parent_name: &str, fanout: usize, breadcrumbs: Vec<usize>, max_depth: usize, flag_set_by: fn(usize, usize) -> bool)
        {
            // Create self meta file.
            let mut self_meta_file = File::create(p.join("self.json")).expect("unable to create self meta file");

            let self_meta_struct = Schema::One(TestUtil::sample_meta_block(Target::Parent, &parent_name, false));
            let self_lines = self_meta_struct.to_serialized_chunk(SchemaFormat::Json);
            writeln!(self_meta_file, "{}", self_lines).expect("unable to write to self meta file");

            let mut item_meta_blocks = vec![];

            for i in 0..fanout {
                let mut new_breadcrumbs = breadcrumbs.clone();

                new_breadcrumbs.push(i);

                let name = if new_breadcrumbs.len() == 0 {
                    String::from("ROOT")
                }
                else {
                    new_breadcrumbs.iter().map(|n| format!("{}", n)).collect::<Vec<_>>().join("_")
                };

                if breadcrumbs.len() >= max_depth {
                    // Create files.
                    let new_path = p.join(&name);
                    File::create(&new_path).expect("unable to create item file");
                } else {
                    // Create dirs and then recurse.
                    let new_path = p.join(&name);
                    db.create(&new_path).expect("unable to create item directory");
                    fill_dir(&new_path, &db, &name, fanout, new_breadcrumbs, max_depth, flag_set_by);
                }

                let include_flag_key = flag_set_by(max_depth - breadcrumbs.len(), i);

                let item_meta_block = TestUtil::sample_meta_block(Target::Siblings, &name, include_flag_key);
                item_meta_blocks.push(item_meta_block);
            }

            // Create item meta file.
            let mut item_meta_file = File::create(p.join("item.json")).expect("unable to create item meta file");

            let item_meta_struct = Schema::Seq(item_meta_blocks);
            let item_lines = item_meta_struct.to_serialized_chunk(SchemaFormat::Json);
            writeln!(item_meta_file, "{}", item_lines).expect("unable to write to item meta file");
        }

        let db = DirBuilder::new();

        fill_dir(root_dir.path(), &db, "ROOT", fanout, vec![], max_depth, flag_set_by);

        std::thread::sleep(Duration::from_millis(1));
        root_dir
    }

    pub fn i(i: i64) -> Value {
        Value::Integer(i)
    }

    pub fn d_raw(i: i64, e: u32) -> Decimal {
        Decimal::new(i.into(), e)
    }

    pub fn d(i: i64, e: u32) -> Value {
        Value::Decimal(Self::d_raw(i, e))
    }

    pub fn s<I: Into<String>>(s: I) -> Value {
        Value::String(s.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_meta_fanout_test_dir() {
        TestUtil::create_meta_fanout_test_dir("create_meta_fanout_test_dir", 3, 3, |_, _| true);
    }

    #[test]
    fn sample_number_sequence() {
        let i = TestUtil::i;
        let d = TestUtil::d;

        let test_cases = vec![
            (
                TestUtil::sample_number_sequence(2, false, false, false),
                Value::Sequence(vec![i(1), i(-1), d(5, 1), d(-5, 1), i(2), i(-2), d(15, 1), d(-15, 1)]),
            ),
            (
                TestUtil::sample_number_sequence(2, true, false, false),
                Value::Sequence(vec![i(1), i(-1), d(5, 1), d(-5, 1), i(2), i(-2), d(15, 1), d(-15, 1), d(25, 1), d(-25, 1)]),
            ),
            (
                TestUtil::sample_number_sequence(2, false, false, true),
                Value::Sequence(vec![i(1), i(-1), d(5, 1), d(-5, 1), i(2), i(-2), d(15, 1), d(-15, 1), i(0)]),
            ),
            (
                TestUtil::sample_number_sequence(2, true, false, true),
                Value::Sequence(vec![i(1), i(-1), d(5, 1), d(-5, 1), i(2), i(-2), d(15, 1), d(-15, 1), d(25, 1), d(-25, 1), i(0)]),
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(input, expected);
        }
    }

    #[test]
    fn to_serialized_chunk() {
        let dec = Decimal::new(31415.into(), 4);

        let seq_a = Value::Sequence(vec![Value::Integer(27), Value::String("string".into())]);
        let seq_b = Value::Sequence(vec![Value::Boolean(false), Value::Null, Value::Decimal(dec)]);

        let seq_seq = Value::Sequence(vec![seq_a.clone(), seq_b.clone()]);

        let map = Value::Mapping(btreemap![
            "key_a".into() => seq_a.clone(),
            "key_b".into() => seq_b.clone(),
            "key_c".into() => seq_seq.clone(),
        ]);

        let inputs_and_expected = vec![
            (
                (seq_a.clone(), SchemaFormat::Json),
                "[\n  27,\n  \"string\"\n]",
            ),
            (
                (seq_a.clone(), SchemaFormat::Yaml),
                "- 27\n- string",
            ),
            (
                (seq_seq.clone(), SchemaFormat::Json),
                "[\n  [\n    27,\n    \"string\"\n  ],\n  [\n    false,\n    null,\n    3.1415\n  ]\n]",
            ),
            (
                (seq_seq.clone(), SchemaFormat::Yaml),
                "- - 27\n  - string\n- - false\n  - ~\n  - 3.1415",
            ),
            (
                (map.clone(), SchemaFormat::Json),
                "{\n  \"key_a\": [\n    27,\n    \"string\"\n  ],\n  \"key_b\": [\n    false,\n    null,\n    3.1415\n  ],\n  \"key_c\": [\n    [\n      27,\n      \"string\"\n    ],\n    [\n      false,\n      null,\n      3.1415\n    ]\n  ]\n}",
            ),
            (
                (map.clone(), SchemaFormat::Yaml),
                "key_a:\n  - 27\n  - string\nkey_b:\n  - false\n  - ~\n  - 3.1415\nkey_c:\n  - - 27\n    - string\n  - - false\n    - ~\n    - 3.1415",
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (mv, schema_format) = inputs;

            let produced = mv.to_serialized_chunk(schema_format);

            assert_eq!(expected, produced);
        }
    }
}