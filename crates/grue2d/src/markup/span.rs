/*!
Span of the simple markup text
*/

use snow2d::gfx::tex::SpriteData;
use thiserror::Error;

use crate::markup::token::*;

/// TODO: report error location
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("UnexpectedMacroTag")]
    UnexpectedMacroTag,
    #[error("{0}")]
    TokenizeError(#[from] TokenizeError),
}

/// View representation of a rather rich text
#[derive(Debug, Clone)]
pub struct TextView<'a> {
    nodes: Vec<Span<'a>>,
}

impl<'a> TextView<'a> {
    pub fn from_tokens(tks: &[Token<'a>]) -> Result<Self, ParseError> {
        let mut nodes = Vec::with_capacity(tks.len());

        for tk in tks {
            let node = match tk {
                Token::Text(text) => Span::Text(TextSpan {
                    slice: text.slice,
                    font_face: FontFace::default(),
                    word_kind: None,
                }),
                Token::Macro(m) => {
                    let word_kind = match m.tag {
                        x if x == "chara" => Some(WordKind::Chara),
                        x if x == "place" => Some(WordKind::Place),
                        x if x == "kwd" => Some(WordKind::Keyword),
                        _ => None,
                    };

                    let font_face = match m.tag {
                        "i" => FontFace::Italic,
                        "b" => FontFace::Bold,
                        _ => FontFace::Regular,
                    };

                    if word_kind.is_none() && font_face == FontFace::Regular {
                        return Err(ParseError::UnexpectedMacroTag);
                    }

                    Span::Text(TextSpan {
                        slice: m.content,
                        font_face,
                        word_kind,
                    })
                }
            };

            nodes.push(node);
        }

        Ok(Self { nodes })
    }
}

#[derive(Debug, Clone)]
pub enum Span<'a> {
    Text(TextSpan<'a>),
    Image(ImageSpan),
}

#[derive(Debug, Clone)]
pub struct TextSpan<'a> {
    slice: &'a str,
    font_face: FontFace,
    word_kind: Option<WordKind>,
}

#[derive(Debug, Clone)]
pub struct ImageSpan {
    img: SpriteData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WordKind {
    Chara,
    Place,
    Keyword,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontFace {
    Regular,
    Italic,
    Bold,
}

impl Default for FontFace {
    fn default() -> Self {
        Self::Regular
    }
}
