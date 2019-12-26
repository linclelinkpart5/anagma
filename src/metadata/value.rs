//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;
use std::convert::TryFrom;

use rust_decimal::Decimal;

use crate::util::Number;

pub type Sequence = Vec<Value>;
pub type Mapping = BTreeMap<String, Value>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash, Deserialize, EnumDiscriminants)]
#[serde(untagged)]
#[strum_discriminants(name(ValueKind))]
pub enum Value {
    Null,
    String(String),
    Sequence(Sequence),
    Mapping(Mapping),
    Integer(i64),
    Boolean(bool),
    Decimal(Decimal),
}

impl Value {
    pub fn get_key_path<S: AsRef<str>>(&self, key_path: &[S]) -> Option<&Self> {
        let mut curr_val = self;

        for key in key_path {
            // See if the current meta value is indeed a mapping.
            match curr_val {
                Self::Mapping(map) => {
                    // See if the current key in the key path is found in this mapping.
                    match map.get(key.as_ref()) {
                        // Unable to proceed on the key path, short circuit.
                        None => return None,

                        // The current key was found, set the new current value.
                        Some(val) => { curr_val = val; }
                    }
                },

                // An attempt was made to get the key of a non-mapping, short circuit.
                _ => return None,
            }
        }

        // The remaining current value is what is needed to return.
        Some(curr_val)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[cfg(test)]
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<Decimal> for Value {
    fn from(d: Decimal) -> Self {
        Self::Decimal(d)
    }
}

impl From<Sequence> for Value {
    fn from(s: Sequence) -> Self {
        Self::Sequence(s)
    }
}

impl From<Mapping> for Value {
    fn from(m: Mapping) -> Self {
        Self::Mapping(m)
    }
}

impl From<Number> for Value {
    fn from(nl: Number) -> Value {
        match nl {
            Number::Integer(i) => Self::from(i),
            Number::Decimal(d) => Self::from(d),
        }
    }
}

impl TryFrom<Value> for Number {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => Ok(Self::from(i)),
            Value::Decimal(d) => Ok(Self::from(d)),
            _ => Err(()),
        }
    }
}

impl<'k> TryFrom<&'k Value> for Number {
    type Error = ();

    fn try_from(value: &'k Value) -> Result<Self, Self::Error> {
        match value {
            &Value::Integer(i) => Ok(Self::Integer(i)),
            &Value::Decimal(d) => Ok(Self::Decimal(d)),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

impl<'k> TryFrom<&'k Value> for bool {
    type Error = ();

    fn try_from(value: &'k Value) -> Result<Self, Self::Error> {
        match value {
            &Value::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for Sequence {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Sequence(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for Mapping {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Mapping(m) => Ok(m),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    use rust_decimal::Decimal;

    #[test]
    fn test_deserialize() {
        let inputs_and_expected = vec![
            ("null", Value::Null),
            (r#""string""#, Value::String(String::from("string"))),
            ("27", Value::Integer(27)),
            ("-27", Value::Integer(-27)),
            ("3.1415", Value::Decimal(Decimal::new(31415.into(), 4))),
            ("-3.1415", Value::Decimal(-Decimal::new(31415.into(), 4))),
            ("true", Value::Boolean(true)),
            ("false", Value::Boolean(false)),
            (
                r#"[null, "string", 27, true]"#,
                Value::Sequence(vec![
                    Value::Null,
                    Value::String(String::from("string")),
                    Value::Integer(27),
                    Value::Boolean(true),
                ]),
            ),
            (
                r#"{"key_a": "string", "key_b": -27, "key_c": false}"#,
                Value::Mapping(btreemap![
                    String::from("key_a") => Value::String(String::from("string")),
                    String::from("key_b") => Value::Integer(-27),
                    String::from("key_c") => Value::Boolean(false),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = serde_json::from_str::<Value>(&input).unwrap();
            assert_eq!(expected, produced);
        }

        let inputs_and_expected = vec![
            ("null", Value::Null),
            ("~", Value::Null),
            (r#""string""#, Value::String(String::from("string"))),
            ("string", Value::String(String::from("string"))),
            ("27", Value::Integer(27)),
            ("-27", Value::Integer(-27)),
            ("3.1415", Value::Decimal(Decimal::new(31415.into(), 4))),
            ("-3.1415", Value::Decimal(-Decimal::new(31415.into(), 4))),
            ("true", Value::Boolean(true)),
            ("false", Value::Boolean(false)),
            (
                r#"[null, "string", 27, true]"#,
                Value::Sequence(vec![
                    Value::Null,
                    Value::String(String::from("string")),
                    Value::Integer(27),
                    Value::Boolean(true),
                ]),
            ),
            (
                "- null\n- string\n- 27\n- true",
                Value::Sequence(vec![
                    Value::Null,
                    Value::String(String::from("string")),
                    Value::Integer(27),
                    Value::Boolean(true),
                ]),
            ),
            (
                r#"{"key_a": "string", "key_b": -27, "key_c": false}"#,
                Value::Mapping(btreemap![
                    String::from("key_a") => Value::String(String::from("string")),
                    String::from("key_b") => Value::Integer(-27),
                    String::from("key_c") => Value::Boolean(false),
                ]),
            ),
            (
                "key_a: string\nkey_b: -27\nkey_c: false",
                Value::Mapping(btreemap![
                    String::from("key_a") => Value::String(String::from("string")),
                    String::from("key_b") => Value::Integer(-27),
                    String::from("key_c") => Value::Boolean(false),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = serde_yaml::from_str::<Value>(&input).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_get_key_path() {
        let key_str_a = String::from("key_a");
        let key_str_b = String::from("key_b");
        let key_str_c = String::from("key_c");
        let key_str_x = String::from("key_x");

        let val_nil = Value::Null;
        let val_str_a = Value::String(String::from("val_a"));
        let val_str_b = Value::String(String::from("val_b"));
        let val_str_c = Value::String(String::from("val_c"));
        let val_seq_a = Value::Sequence(vec![
            val_str_a.clone(), val_str_a.clone(), val_str_a.clone(),
        ]);
        let val_seq_b = Value::Sequence(vec![
            val_str_b.clone(), val_str_b.clone(), val_str_b.clone(),
        ]);
        let val_seq_c = Value::Sequence(vec![
            val_str_c.clone(), val_str_c.clone(), val_str_c.clone(),
        ]);
        let val_map_a = Value::Mapping(btreemap![
            key_str_a.clone() => val_str_a.clone(),
            key_str_b.clone() => val_str_b.clone(),
            key_str_c.clone() => val_str_c.clone(),
        ]);
        let val_map_b = Value::Mapping(btreemap![
            key_str_a.clone() => val_seq_a.clone(),
            key_str_b.clone() => val_seq_b.clone(),
            key_str_c.clone() => val_seq_c.clone(),
        ]);
        let val_map_c = Value::Mapping(btreemap![
            key_str_a.clone() => val_nil.clone(),
            key_str_b.clone() => val_nil.clone(),
            key_str_c.clone() => val_nil.clone(),
        ]);
        let val_map_d = Value::Mapping(btreemap![
            key_str_a.clone() => val_map_a.clone(),
            key_str_b.clone() => val_map_b.clone(),
            key_str_c.clone() => val_map_c.clone(),
        ]);

        let inputs_and_expected = vec![

            // An empty key path always returns the original value.
            ((&val_nil, vec![]), Some(&val_nil)),
            ((&val_str_a, vec![]), Some(&val_str_a)),
            ((&val_seq_a, vec![]), Some(&val_seq_a)),
            ((&val_map_a, vec![]), Some(&val_map_a)),

            // A non-empty key path returns no value on non-maps.
            ((&val_nil, vec![key_str_a.clone()]), None),
            ((&val_str_a, vec![key_str_a.clone()]), None),
            ((&val_seq_a, vec![key_str_a.clone()]), None),

            // If the key is not found in a mapping, nothing is returned.
            ((&val_map_a, vec![key_str_x.clone()]), None),
            ((&val_map_d, vec![key_str_a.clone(), key_str_x.clone()]), None),

            // Positive test cases.
            ((&val_map_a, vec![key_str_a.clone()]), Some(&val_str_a)),
            ((&val_map_b, vec![key_str_a.clone()]), Some(&val_seq_a)),
            ((&val_map_c, vec![key_str_a.clone()]), Some(&val_nil)),
            ((&val_map_d, vec![key_str_a.clone()]), Some(&val_map_a)),
            ((&val_map_a, vec![key_str_b.clone()]), Some(&val_str_b)),
            ((&val_map_b, vec![key_str_b.clone()]), Some(&val_seq_b)),
            ((&val_map_c, vec![key_str_b.clone()]), Some(&val_nil)),
            ((&val_map_d, vec![key_str_b.clone()]), Some(&val_map_b)),
            ((&val_map_a, vec![key_str_c.clone()]), Some(&val_str_c)),
            ((&val_map_b, vec![key_str_c.clone()]), Some(&val_seq_c)),
            ((&val_map_c, vec![key_str_c.clone()]), Some(&val_nil)),
            ((&val_map_d, vec![key_str_c.clone()]), Some(&val_map_c)),

            // Nested positive test cases.
            ((&val_map_d, vec![key_str_a.clone(), key_str_a.clone()]), Some(&val_str_a)),
            ((&val_map_d, vec![key_str_b.clone(), key_str_b.clone()]), Some(&val_seq_b)),
            ((&val_map_d, vec![key_str_c.clone(), key_str_c.clone()]), Some(&val_nil)),

        ];

        for (input, expected) in inputs_and_expected {
            let (val, key_path) = input;
            let produced = val.get_key_path(&key_path);
            assert_eq!(expected, produced);
        }
    }
}
