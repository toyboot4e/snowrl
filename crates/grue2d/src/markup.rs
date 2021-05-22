/*!
Simple markup language
*/

pub mod span;
pub mod token;

use span::{ParseError, TextView};
use token::{Token, Tokenizer};

pub fn to_spans<'a>(src: &'a str) -> Result<(Vec<Token<'a>>, TextView<'a>), ParseError> {
    let tks = Tokenizer::tokenize(src)?;
    let view = TextView::from_tokens(&tks)?;
    Ok((tks, view))
}
