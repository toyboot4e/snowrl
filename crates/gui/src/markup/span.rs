/*!
Span with semantic information as a specific DSL for SnowRL
*/

use thiserror::Error;

use snow2d::input::Key;

use crate::markup::token::*;

/// `&Token` -> `SpanLines`
pub fn to_spans<'a>(tks: &[Token<'a>]) -> Result<SpanLines<'a>, ParseError> {
    SpanLines::from_tokens(&tks)
}

/// TODO: report error location
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("UnexpectedMacroTag")]
    UnexpectedMacroTag,
    #[error("{0}")]
    TokenizeError(#[from] TokenizeError),
    #[error("{0}")]
    InvalidIconArg(String),
}

#[derive(Debug, Clone)]
pub struct LineSpan {
    pub lo: usize,
    pub hi: usize,
}

/// Spans of a rather rich-text
#[derive(Debug, Clone)]
pub struct SpanLines<'a> {
    spans: Vec<Span<'a>>,
    lines: Vec<LineSpan>,
}

impl<'a> SpanLines<'a> {
    pub fn lines(&self) -> Vec<&[Span<'a>]> {
        let mut lines = Vec::with_capacity(1 + self.spans.len() / 4);

        for line in &self.lines {
            lines.push(&self.spans[line.lo..line.hi]);
        }

        lines
    }

    pub fn line_spans(&self) -> &[LineSpan] {
        &self.lines
    }

    fn from_tokens(tks: &[Token<'a>]) -> Result<Self, ParseError> {
        let mut spans = Vec::with_capacity(tks.len());
        let mut nls = Vec::with_capacity(4);

        for tk in tks {
            let node = match tk {
                Token::Newline => {
                    nls.push(spans.len());
                    continue;
                }
                Token::Text(text) => Span::Text(TextSpan {
                    slice: text.slice,
                    font_face: FontFace::default(),
                    word_kind: None,
                }),
                Token::Macro(m) => match m.tag {
                    "kbd" => Span::Kbd(KbdSpan::parse(m.content)?),
                    _ => {
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
                },
            };

            spans.push(node);
        }

        let lines = if let Some(first) = nls.first() {
            let mut lines = Vec::new();

            lines.push(LineSpan { lo: 0, hi: *first });

            for i in 0..(nls.len() - 1) {
                lines.push(LineSpan {
                    lo: nls[i],
                    hi: nls[i + 1],
                });
            }

            if let Some(last) = nls.last() {
                lines.push(LineSpan {
                    lo: last.clone(),
                    hi: spans.len(),
                });
            }

            lines
        } else {
            vec![LineSpan {
                lo: 0,
                hi: tks.len(),
            }]
        };

        Ok(Self { spans, lines })
    }
}

#[derive(Debug, Clone)]
pub enum Span<'a> {
    Text(TextSpan<'a>),
    Image(ImageSpan<'a>),
    Kbd(KbdSpan),
}

/// Text with decoration
#[derive(Debug, Clone)]
pub struct TextSpan<'a> {
    pub slice: &'a str,
    pub font_face: FontFace,
    pub word_kind: Option<WordKind>,
}

/// Image
#[derive(Debug, Clone)]
pub struct ImageSpan<'a> {
    pub data: &'a str,
    // align, scale, rotation, etc.
}

/// Keyboard image
#[derive(Debug, Clone)]
pub struct KbdSpan {
    pub keys: Vec<Key>,
}

impl KbdSpan {
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let keys = s
            .chars()
            .map(|c| Key::from_char(c).ok_or_else(|| ParseError::InvalidIconArg(s.to_string())))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { keys })
    }
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
