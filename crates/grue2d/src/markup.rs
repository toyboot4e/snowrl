/*!
Simple markup language
*/

pub mod token;
pub mod view;

use token::{Token, Tokenizer};
use view::{ParseError, TextView};

pub fn parse<'a>(src: &'a str) -> Result<(Vec<Token<'a>>, TextView<'a>), ParseError> {
    let tks = Tokenizer::tokenize(src)?;
    let view = TextView::from_tokens(&tks)?;
    Ok((tks, view))
}
