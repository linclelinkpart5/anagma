#![cfg(test)]

mod entry;

use std::fs::DirBuilder;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use rand::seq::SliceRandom;
use rust_decimal::Decimal;
use tempfile::Builder;
use tempfile::TempDir;

use maplit::btreemap;
use rust_decimal_macros::dec;

use crate::metadata::block::Block;
use crate::metadata::schema::Schema;
use crate::metadata::target::Target;
use crate::metadata::value::Value;
use crate::metadata::value::Sequence;
use crate::metadata::value::Mapping;

use self::entry::DEFAULT_FLAGGER;
use self::entry::DEFAULT_LIBRARY;

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
            let self_meta_file = File::create(p.join("self.json")).expect("unable to create self meta file");

            let self_meta_struct = Schema::One(TestUtil::sample_meta_block(Target::Parent, &parent_name, false));
            serde_json::to_writer_pretty(self_meta_file, &self_meta_struct).unwrap();

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
            let item_meta_file = File::create(p.join("item.json")).expect("unable to create item meta file");

            let item_meta_struct = Schema::Seq(item_meta_blocks);
            serde_json::to_writer_pretty(item_meta_file, &item_meta_struct).unwrap();
        }

        let db = DirBuilder::new();

        fill_dir(root_dir.path(), &db, "ROOT", fanout, vec![], max_depth, flag_set_by);

        std::thread::sleep(Duration::from_millis(1));
        root_dir
    }

    pub fn i(i: i64) -> Value {
        Value::Integer(i)
    }

    pub fn d(d: Decimal) -> Value {
        Value::Decimal(d)
    }

    pub fn s<I: Into<String>>(s: I) -> Value {
        Value::String(s.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::TestUtil as TU;

    use rust_decimal_macros::dec;

    #[test]
    fn create_meta_fanout_test_dir() {
        TestUtil::create_meta_fanout_test_dir("create_meta_fanout_test_dir", 3, 3, |_, _| true);
    }

    #[test]
    fn sample_number_sequence() {
        let test_cases = vec![
            (
                TestUtil::sample_number_sequence(2, false, false, false),
                Value::Sequence(vec![
                    TU::i(1), TU::i(-1), TU::d(dec!(0.5)), TU::d(dec!(-0.5)),
                    TU::i(2), TU::i(-2), TU::d(dec!(1.5)), TU::d(dec!(-1.5)),
                ]),
            ),
            (
                TestUtil::sample_number_sequence(2, true, false, false),
                Value::Sequence(vec![
                    TU::i(1), TU::i(-1), TU::d(dec!(0.5)), TU::d(dec!(-0.5)),
                    TU::i(2), TU::i(-2), TU::d(dec!(1.5)), TU::d(dec!(-1.5)),
                    TU::d(dec!(2.5)), TU::d(dec!(-2.5)),
                ]),
            ),
            (
                TestUtil::sample_number_sequence(2, false, false, true),
                Value::Sequence(vec![
                    TU::i(1), TU::i(-1), TU::d(dec!(0.5)), TU::d(dec!(-0.5)),
                    TU::i(2), TU::i(-2), TU::d(dec!(1.5)), TU::d(dec!(-1.5)),
                    TU::i(0),
                ]),
            ),
            (
                TestUtil::sample_number_sequence(2, true, false, true),
                Value::Sequence(vec![
                    TU::i(1), TU::i(-1), TU::d(dec!(0.5)), TU::d(dec!(-0.5)),
                    TU::i(2), TU::i(-2), TU::d(dec!(1.5)), TU::d(dec!(-1.5)),
                    TU::d(dec!(2.5)), TU::d(dec!(-2.5)), TU::i(0),
                ]),
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(input, expected);
        }
    }
}
