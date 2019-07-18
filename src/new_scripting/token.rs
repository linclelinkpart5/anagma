use crate::metadata::types::MetaVal;

const OPERATOR_SIGIL: &'static str = "$";
const OPERATOR_SIGIL_ESCAPE: &'static str = "$$";

#[derive(Deserialize)]
pub struct RawTokenList(Vec<MetaVal>);

impl RawTokenList {
    fn into_token_list(self) -> Vec<Token> {
        let mut tokens = Vec::with_capacity(self.0.len());

        for mv in self.0.into_iter() {
            let token =
                match mv {
                    MetaVal::Str(s) => {
                        if s.starts_with(OPERATOR_SIGIL_ESCAPE) {
                            // Replace the sigil escape with just the sigil, and treat as a string.
                            Token::Value(MetaVal::Str(s.replacen(OPERATOR_SIGIL_ESCAPE, OPERATOR_SIGIL, 1)))
                        }
                        else if s.starts_with(OPERATOR_SIGIL) {
                            // Actually an operator, process as such.
                            Token::Value(MetaVal::Nil)
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

        tokens
    }
}

/// Represents the various values found when parsing a script command.
pub enum Token {
    Value(MetaVal),
}
