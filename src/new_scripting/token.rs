use std::str::FromStr;
use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::new_scripting::operators::UnaryOp;
use crate::new_scripting::operators::BinaryOp;

const UNARY_OP_SIGIL: &'static str = "$";
const BINARY_OP_SIGIL: &'static str = "@";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidUnaryOp(String),
    InvalidBinaryOp(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::InvalidUnaryOp(ref s) => write!(f, "invalid unary operator name: {}", s),
            Self::InvalidBinaryOp(ref s) => write!(f, "invalid binary operator name: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct RawValues(Vec<MetaVal>);

impl TryFrom<RawValues> for Vec<Token> {
    type Error = Error;

    fn try_from(raw_values: RawValues) -> Result<Self, Self::Error> {
        let mut tokens = Vec::with_capacity(raw_values.0.len());

        for mv in raw_values.0.into_iter() {
            let token =
                match mv {
                    MetaVal::Str(s) => {
                        if s.starts_with(UNARY_OP_SIGIL) {
                            // Trim the first occurrence of the sigil.
                            let s_trimmed = s.replacen(UNARY_OP_SIGIL, "", 1);

                            // If the trimmed string still starts with a sigil, it was an escaped sigil, treat as a string.
                            if s_trimmed.starts_with(UNARY_OP_SIGIL) {
                                Token::Value(MetaVal::Str(s_trimmed))
                            }
                            else {
                                // Actually an operator, process as such.
                                let unary_op = UnaryOp::from_str(&s_trimmed).map_err(|_| Error::InvalidUnaryOp(s_trimmed))?;
                                Token::UnaryOp(unary_op)
                            }
                        }
                        else if s.starts_with(BINARY_OP_SIGIL) {
                            // Trim the first occurrence of the sigil.
                            let s_trimmed = s.replacen(BINARY_OP_SIGIL, "", 1);

                            // If the trimmed string still starts with a sigil, it was an escaped sigil, treat as a string.
                            if s_trimmed.starts_with(BINARY_OP_SIGIL) {
                                Token::Value(MetaVal::Str(s_trimmed))
                            }
                            else {
                                // Actually an operator, process as such.
                                let binary_op = BinaryOp::from_str(&s_trimmed).map_err(|_| Error::InvalidBinaryOp(s_trimmed))?;
                                Token::BinaryOp(binary_op)
                            }
                        }
                        else {
                            // A plain string that doesn't start with any sigils.
                            Token::Value(MetaVal::Str(s))
                        }
                    },
                    mv => {
                        // Take the original meta value and wrap it in a token.
                        Token::Value(mv)
                    },
                }
            ;

            tokens.push(token);
        }

        Ok(tokens)
    }
}

/// Represents the various values found when parsing a script command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Value(MetaVal),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_values_deserialize() {
        let raw_values_ser = r#"[1, 3.1415, false, null, "hello", "$collect", "@all", "$$escaped", "@@escaped"]"#;
        let raw_values: RawValues = serde_json::from_str(raw_values_ser).unwrap();

        println!("{:?}", raw_values);

        let expected = vec![
            MetaVal::Int(1),
            MetaVal::Dec(dec!(3.1415)),
            MetaVal::Bul(false),
            MetaVal::Nil,
            MetaVal::Str("hello".into()),
            MetaVal::Str("$collect".into()),
            MetaVal::Str("@all".into()),
            MetaVal::Str("$$escaped".into()),
            MetaVal::Str("@@escaped".into()),
        ];
        let produced = raw_values.0;

        assert_eq!(expected, produced);
    }

    #[test]
    fn raw_values_try_into_vec_token() {
        let raw_values = RawValues(vec![
            MetaVal::Int(1),
            MetaVal::Dec(dec!(3.1415)),
            MetaVal::Bul(false),
            MetaVal::Nil,
            MetaVal::Str("hello".into()),
            MetaVal::Str("$collect".into()),
            MetaVal::Str("@all".into()),
            MetaVal::Str("$$escaped".into()),
            MetaVal::Str("@@escaped".into()),
        ]);

        let expected = Ok(vec![
            Token::Value(MetaVal::Int(1)),
            Token::Value(MetaVal::Dec(dec!(3.1415))),
            Token::Value(MetaVal::Bul(false)),
            Token::Value(MetaVal::Nil),
            Token::Value(MetaVal::Str("hello".into())),
            Token::UnaryOp(UnaryOp::Collect),
            Token::BinaryOp(BinaryOp::All),
            Token::Value(MetaVal::Str("$escaped".into())),
            Token::Value(MetaVal::Str("@escaped".into())),
        ]);
        let produced: Result<Vec<Token>, _> = TryFrom::try_from(raw_values);

        assert_eq!(expected, produced);

        let raw_values = RawValues(vec![
            MetaVal::Str("$UNKNOWN!".into()),
            MetaVal::Int(2),
            MetaVal::Dec(dec!(2.7182)),
            MetaVal::Bul(true),
            MetaVal::Nil,
            MetaVal::Str("goodbye".into()),
        ]);

        let expected = Err(Error::InvalidUnaryOp("UNKNOWN!".into()));
        let produced: Result<Vec<Token>, _> = TryFrom::try_from(raw_values);

        assert_eq!(expected, produced);

        let raw_values = RawValues(vec![
            MetaVal::Str("@UNKNOWN!".into()),
            MetaVal::Int(2),
            MetaVal::Dec(dec!(2.7182)),
            MetaVal::Bul(true),
            MetaVal::Nil,
            MetaVal::Str("goodbye".into()),
        ]);

        let expected = Err(Error::InvalidBinaryOp("UNKNOWN!".into()));
        let produced: Result<Vec<Token>, _> = TryFrom::try_from(raw_values);

        assert_eq!(expected, produced);
    }
}
