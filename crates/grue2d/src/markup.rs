/*!
Simple markup language
*/

use thiserror::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ByteSpan {
    lo: usize,
    hi: usize,
}

impl ByteSpan {
    pub fn slice<'a>(&self, src: &'a str) -> Option<&'a str> {
        if self.lo < self.hi {
            Some(&src[self.lo..self.hi])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    /// Example: `:italic[text]`
    Macro(MacroToken<'a>),
    /// Ordinary text
    Text(TextToken<'a>),
}

/// `:marco[text]`
#[derive(Debug, Clone, PartialEq)]
pub struct MacroToken<'a> {
    tag: &'a str,
    content: &'a str,
}

/// Ordinary text
#[derive(Debug, Clone, PartialEq)]
pub struct TextToken<'a> {
    slice: &'a str,
}

#[derive(Debug, Clone, Error)]
pub enum TokenizeError {
    #[error("Unable to find `[` of a macro")]
    UnableToFindMacroOpenBracket,
    #[error("Unable to find `]` of a macro")]
    UnableToFindMacroCloseBracket,
    #[error("Unexpected order of `[` and `]` for a macro")]
    UnexpectdOrderOfMacroBrackets,
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    /// UTF-8 string slice referenced as bytes
    src: &'a [u8],
    sp: ByteSpan,
    tks: Vec<Token<'a>>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a one-shot toknizer and runs it
    pub fn tokenize(src: &'a str) -> Result<Vec<Token<'a>>, TokenizeError> {
        let mut me = Self::new(src);
        me.tokenize_impl()?;
        Ok(me.into_tokens())
    }

    /// Returns multi-shot tokenizer
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            sp: ByteSpan { lo: 0, hi: 0 },
            tks: Vec::with_capacity(16),
        }
    }

    fn into_tokens(self) -> Vec<Token<'a>> {
        self.tks
    }

    fn tokenize_impl(&mut self) -> Result<(), TokenizeError> {
        while self.sp.hi < self.src.len() {
            if self.src[self.sp.hi] == b':' {
                let colon = self.sp.hi;

                if colon == 0 || colon != 0 && self.src[colon - 1] == b' ' {
                    self.consume_span();

                    // word starting with `:` is always a macro
                    let src = &self.src[self.sp.hi..];

                    // find `[` and `]`
                    let open = src
                        .iter()
                        .position(|x| *x == b'[')
                        .ok_or(TokenizeError::UnableToFindMacroOpenBracket)?;

                    let close = src
                        .iter()
                        .position(|x| *x == b']')
                        .ok_or(TokenizeError::UnableToFindMacroCloseBracket)?;

                    if !open < close {
                        return Err(TokenizeError::UnexpectdOrderOfMacroBrackets);
                    }

                    let src = unsafe { std::str::from_utf8_unchecked(src) };
                    self.tks.push(Token::Macro(MacroToken {
                        tag: &src[1..open],
                        content: &src[(open + 1)..close],
                    }));

                    self.sp.hi += close + 1;
                    self.sp.lo = self.sp.hi;
                    continue;
                }
            }

            self.sp.hi += 1;
        }

        self.consume_span();

        Ok(())
    }

    /// Consumes accumulates span as [`TextToken`]
    fn consume_span(&mut self) {
        let src = unsafe { std::str::from_utf8_unchecked(&self.src) };
        if let Some(slice) = self.sp.slice(src) {
            self.tks.push(Token::Text(TextToken { slice }));
        }

        self.sp.lo = self.sp.hi;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        let src = "aa   :macro[aaaaaaaa] end";
        //         0         1         2         3
        //         0 2 4 6 8 0 2 4 6 8 0 2 4 6 8 0
        let tks = Tokenizer::tokenize(src).unwrap();

        assert_eq!(&tks[0], &Token::Text(TextToken { slice: &src[0..5] }));

        assert_eq!(
            &tks[1],
            &Token::Macro(MacroToken {
                tag: &src[6..11],
                content: &src[12..20],
            })
        );

        assert_eq!(&tks[2], &Token::Text(TextToken { slice: &src[21..] }));
    }
}
