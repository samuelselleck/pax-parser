use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::fmt::Write;

use crate::{
    lexer::{Span, SrcToken, Token},
    Parser,
};

impl<'src> Parser<'src> {
    pub fn expect(&mut self, token: Token) -> Result<SrcToken, PaxParseError> {
        self.tokens
            .expect_token(token)
            .map_err(|e| self.expected_with_span(e.token_found.span, [e.token_expected_type]))
    }

    pub fn expect_sequence<const N: usize>(
        &mut self,
        expected_types: [Token; N],
    ) -> Result<[SrcToken; N], PaxParseError> {
        self.tokens
            .expect_token_sequence(expected_types)
            .map_err(|e| self.expected_with_span(e.token_found.span, [e.token_expected_type]))
    }

    pub fn error<const N: usize>(&mut self, tokens: [Token; N]) -> PaxParseError {
        let tok = self.tokens.next();
        self.expected_with_span(tok.span, tokens)
    }

    fn expected_with_span<const N: usize>(&self, span: Span, tokens: [Token; N]) -> PaxParseError {
        // TODO add info from self.context
        let mut expect_str = String::from("expected ");
        match tokens.len() {
            0 => {
                write!(expect_str, "<unspecificed>").unwrap();
            }
            1 => {
                write!(expect_str, "{}", tokens[0]).unwrap();
            }
            2.. => {
                let last = tokens.len() - 1;
                for t in &tokens[0..last] {
                    write!(expect_str, "{}, ", t).unwrap();
                }
                // remove the last ", " part
                expect_str.pop();
                expect_str.pop();
                write!(expect_str, " or {}", tokens[last]).unwrap();
            }
        }
        PaxParseError::new("unexpected character(s)").annotation(span, expect_str)
    }
}

#[derive(Debug)]
pub struct Annotation {
    span: Span,
    text: String,
    annotation_type: AnnotationType,
}

#[derive(Debug)]
pub enum AnnotationType {
    Primary,
    Secondary,
}

#[derive(Debug)]
pub struct PaxParseError {
    _error_code: usize,
    short_description: String,
    annotations: Vec<Annotation>,
    help: Option<String>,
}

impl PaxParseError {
    pub fn new(short_description: impl Into<String>) -> Self {
        Self {
            // TODO fill in error codes
            _error_code: 0,
            short_description: short_description.into(),
            annotations: Vec::new(),
            help: None,
        }
    }

    pub fn annotation(mut self, span: Span, text: impl Into<String>) -> Self {
        self.annotations.push(Annotation {
            annotation_type: match self.annotations.is_empty() {
                true => AnnotationType::Primary,
                false => AnnotationType::Secondary,
            },
            span,
            text: text.into(),
        });
        self
    }

    pub fn print_with_file(
        &self,
        file_name: &str,
        file: &str,
    ) -> Result<(), codespan_reporting::files::Error> {
        let mut files = SimpleFiles::new();
        let file_id = files.add(file_name, file);
        let diagnostic = Diagnostic::error()
            .with_message(&self.short_description)
            .with_code("E")
            .with_labels(
                self.annotations
                    .iter()
                    .map(|a| match a.annotation_type {
                        AnnotationType::Primary => {
                            Label::primary(file_id, a.span).with_message(&a.text)
                        }
                        AnnotationType::Secondary => {
                            Label::secondary(file_id, a.span).with_message(&a.text)
                        }
                    })
                    .collect(),
            )
            .with_notes(self.help.as_slice().to_vec());

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        term::emit(&mut writer.lock(), &config, &files, &diagnostic)?;
        Ok(())
    }
}
