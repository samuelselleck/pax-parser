use std::{collections::VecDeque, str::CharIndices};
mod span;
mod token;
pub use span::Span;
pub use token::Token;

use crate::utils::MultiPeek;

#[derive(Debug)]
pub struct UnexpectedToken {
    pub token_found: SrcToken,
    pub token_expected_type: Token,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SrcToken {
    pub span: Span,
    pub token_type: Token,
}

pub struct TokenIterator<'src> {
    itr: MultiPeek<CharIndices<'src>>,
    pub src: &'src str,
    peeked: VecDeque<SrcToken>,
    newline_indicies: Vec<usize>,
}

impl<'src> TokenIterator<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            itr: MultiPeek::new(source.char_indices()),
            src: source,
            peeked: VecDeque::new(),
            newline_indicies: Vec::new(),
        }
    }

    pub fn peek_nth(&mut self, i: usize) -> Token {
        while self.peeked.len() <= i {
            let next = self.next_ignore_peeked();
            self.peeked.push_back(next);
        }
        self.peeked[i].token_type
    }

    pub fn peek(&mut self) -> Token {
        self.peek_nth(0)
    }

    pub fn next_if(&mut self, f: impl FnOnce(Token) -> bool) -> Option<SrcToken> {
        let next = self.peek();
        f(next).then(|| self.next())
    }

    pub fn expect_token(&mut self, expected_type: Token) -> Result<SrcToken, UnexpectedToken> {
        let next = self.next();
        if next.token_type == expected_type {
            Ok(next)
        } else {
            Err(UnexpectedToken {
                token_found: next,
                token_expected_type: expected_type,
            })
        }
    }

    pub fn expect_token_sequence<const N: usize>(
        &mut self,
        expected_types: [Token; N],
    ) -> Result<[SrcToken; N], UnexpectedToken> {
        let mut values = [SrcToken::default(); N];
        for i in 0..N {
            values[i] = self.expect_token(expected_types[i])?;
        }
        Ok(values)
    }

    pub fn next(&mut self) -> SrcToken {
        if let Some(front) = self.peeked.pop_front() {
            return front;
        }
        self.next_ignore_peeked()
    }

    fn next_ignore_peeked(&mut self) -> SrcToken {
        //trim all white spaces
        while let Some((i, whitespace)) = self.itr.next_if(|(_, c)| c.is_whitespace()) {
            if whitespace == '\n' {
                self.newline_indicies.push(i);
            }
        }

        let Some((_, char)) = self.itr.peek().cloned() else {
            let start = self.src.len() - 1;
            let end = self.src.len();
            return SrcToken {
                span: Span { start, end },
                token_type: Token::EOF,
            };
        };
        match char {
            c if c.is_alphabetic() => self.ident_like(),
            v if v.is_ascii_digit() => self.number(),
            '(' => self.single(Token::OpenParenth),
            ')' => self.single(Token::CloseParenth),
            '[' => self.single(Token::OpenSquareBrack),
            ']' => self.single(Token::CloseSquareBrack),
            '{' => self.single(Token::OpenCurlBrack),
            '}' => self.single(Token::CloseCurlBrack),
            '*' => self.single(Token::Asterisk),
            '+' => self.single(Token::Plus),
            '-' => self.single(Token::Minus),
            '@' => self.single(Token::AtSymbol),
            ':' => self.one_or_two_if_with(Token::Colon, ':', Token::PathSep),
            ',' => self.single(Token::Comma),
            '#' => self.single(Token::Hashtag),
            '^' => self.single(Token::Exp),
            '/' => self.comment_or_slash(),
            '"' => self.string(),
            '.' => self.one_or_two_if_with(Token::Period, '.', Token::Range),
            '%' => self.one_or_two_if_with(Token::Percent, '%', Token::Remainder),
            '<' => self.one_or_two_if_with(Token::OpenAngBrack, '=', Token::LessOrEq),
            '>' => self.one_or_two_if_with(Token::CloseAngBrack, '=', Token::MoreOrEq),
            '=' => self.one_or_two_if_with(Token::Assign, '=', Token::Eq),
            '!' => self.one_or_two_if_with(Token::Not, '=', Token::NotEq),
            '|' => self.one_or_two_if_with(Token::VertLine, '|', Token::Or),
            '&' => self.one_or_two_if_with(Token::Ampersand, '&', Token::And),
            c => {
                eprintln!("unknown token: {:?}", c);
                self.single(Token::Unknown)
            }
        }
    }

    fn single(&mut self, token_type: Token) -> SrcToken {
        let Some((start, c)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };
        let end = start + c.len_utf8();
        return SrcToken {
            span: Span { start, end },
            token_type,
        };
    }

    fn ident_like(&mut self) -> SrcToken {
        let Some((start, _)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };
        let mut end = start;

        while let Some((i, _)) = self
            .itr
            .next_if(|(_, c)| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == &'_')
        {
            end = i;
        }
        end += 1;
        let src = &self.src[start..end];

        // Trim "self." and "this." in the entire file,
        // instead returning the identifier directly afterwards
        if matches!(src, "self" | "this") {
            if self.itr.next_if(|&(_, c)| c == '.').is_some() {
                // read the next ident instead
                return self.ident_like();
            }
        }

        let token_type = match src {
            "for" => Token::For,
            "if" => Token::If,
            "slot" => Token::Slot,
            "in" => Token::In,
            "px" => Token::Pixels,
            "deg" => Token::Degrees,
            "rad" => Token::Radians,
            "bind" => Token::Bind,
            _ => Token::Identifier,
        };
        return SrcToken {
            span: Span { start, end },
            token_type,
        };
    }

    fn number(&mut self) -> SrcToken {
        let Some((start, _)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };

        let mut end = start;
        while let Some((i, _)) = self.itr.next_if(|(_, c)| c.is_ascii_digit()) {
            end = i;
        }
        let is_float_next = self.itr.peek().is_some_and(|&(_, c)| c == '.')
            && self
                .itr
                .peek_nth(1)
                .is_some_and(|&(_, c)| c.is_ascii_digit());

        if is_float_next {
            // skip '.'
            self.itr.next();
            while let Some((i, _)) = self.itr.next_if(|(_, c)| c.is_ascii_digit()) {
                end = i;
            }
        }
        end += 1;
        SrcToken {
            span: Span { start, end },
            token_type: if is_float_next {
                Token::Float
            } else {
                Token::Integer
            },
        }
    }

    fn comment_or_slash(&mut self) -> SrcToken {
        let Some((start, _)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };

        let mut end = start;
        let token_type = match self.itr.peek() {
            Some(&(_, '/')) => {
                while let Some((i, _)) = self.itr.next_if(|&(_, c)| c != '\n') {
                    end = i;
                }
                Token::Comment
            }
            _ => Token::Slash,
        };
        end += 1;

        SrcToken {
            span: Span { start, end },
            token_type,
        }
    }

    fn one_or_two_if_with(&mut self, if_one: Token, if_with: char, if_two: Token) -> SrcToken {
        let Some((start, character)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };
        let (token_type, end) = if let Some((i, _)) = self.itr.next_if(|&(_, c)| c == if_with) {
            (if_two, i)
        } else {
            (if_one, start + character.len_utf8())
        };
        SrcToken {
            span: Span { start, end },
            token_type,
        }
    }

    fn string(&mut self) -> SrcToken {
        let Some((start, _)) = self.itr.next() else {
            unreachable!("already peeked to match on this function")
        };

        let mut end = start;
        let mut esc = false;
        loop {
            let Some((i, c)) = self.itr.next() else { break };
            end = i;
            match c {
                '\\' => {
                    esc = !esc;
                    continue;
                }
                '"' => {
                    if !esc {
                        break;
                    }
                }
                _ => (),
            };
            esc = false;
        }
        end += 1;

        SrcToken {
            span: Span { start, end },
            token_type: Token::String,
        }
    }
}
