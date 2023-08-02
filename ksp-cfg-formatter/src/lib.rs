//! Parser and formatter for Kerbal Space Program config files, including ModuleManager syntax
#![warn(missing_docs)]
/// Contains code to interface with TypeScript
#[cfg(target_family = "wasm")]
pub mod wasm_bindings;

pub mod parser;

use std::fmt::Display;

use self::parser::document::Document;
use parser::{node::NodeParseError, ASTPrint, Grammar, Rule};
use pest::Parser;
// use reader::{Grammar, Rule};

/// Defines which End of Line sequence to be used
///
/// Can have the values `LF`, `CRLF` or `Identify`.
///
/// When using `Identify`, the formatter tries to figure out what sequence to use, based on the provided text.
///
/// Example:
/// ```
/// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
///
/// let line_return = LineReturn::LF;
///
/// let indentation = Indentation::Tabs;
/// let formatter = Formatter::new(indentation, false, line_return);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum LineReturn {
    /// Line Feed. Used on Linux
    LF,
    /// Carriage Return Line Feed. used on Windows
    CRLF,
    /// The formatter identifies which sequence to use, based on the text
    Identify,
}

/// Indent using `Tabs` or `Spaces(usize)`.
///
/// When using spaces, the number provided is used as each level of indentation
///
/// Example:
/// ```
/// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Spaces(4);
///
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false, line_return);
/// ```
#[derive(Clone, Copy)]
pub enum Indentation {
    /// Number of spaces to indent with
    Spaces(usize),
    /// Used to indicate to indent with tabs
    Tabs,
}

impl std::fmt::Display for Indentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Spaces(n) => write!(f, "{}", " ".repeat(n)),
            Self::Tabs => write!(f, "\t"),
        }
    }
}

impl From<Option<usize>> for Indentation {
    fn from(setting: Option<usize>) -> Self {
        setting.map_or(Self::Tabs, Self::Spaces)
    }
}

/// Struct for holding the settings to use for formatting. use `self.format_text()` to format text
///
/// Example:
/// ```
/// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Tabs;
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false, line_return);
/// # // this is needed to test the code, but not important to readers
/// # let input = String::new();
/// let output = formatter.format_text(&input);
/// ```
///
/// See [`Formatter::format_text()`]
pub struct Formatter {
    indentation: Indentation,
    inline: bool,
    line_return: LineReturn,
}

impl Formatter {
    /// Constructs a new `Formatter` with the settings provided.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let formatter = Formatter::new(Indentation::Tabs, false, LineReturn::Identify);
    /// ```
    #[must_use]
    pub const fn new(indentation: Indentation, inline: bool, line_return: LineReturn) -> Self {
        Self {
            indentation,
            inline,
            line_return,
        }
    }

    /// Takes the provided text and formats it according to the settings of the `Formatter`
    ///
    /// If the formatting fails, the orginal text is returned unchanged
    /// FIXME: This is not indicated in any way
    ///
    /// TODO: Explain the parts of the formatter.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let indentation = Indentation::Tabs;
    /// let line_return = LineReturn::Identify;
    /// let formatter = Formatter::new(indentation, false, line_return);
    /// # // this is needed to test the code, but not important to readers
    /// # let input = String::new();
    /// let output = formatter.format_text(&input);
    /// ```
    #[must_use]
    pub fn format_text(&self, text: &str) -> String {
        ast_format(text, self).map_or(text.to_string(), |res| res)
    }
}

fn ast_format(text: &str, settings: &Formatter) -> Result<String, AstParseError> {
    let use_crlf = if matches!(settings.line_return, LineReturn::Identify) {
        text.contains("\r\n")
    } else {
        matches!(settings.line_return, LineReturn::CRLF)
    };
    let document = Grammar::parse(Rule::document, text)?.next().unwrap();
    let parsed_document = Document::try_from(document)?;
    let line_ending = if use_crlf { "\r\n" } else { "\n" };
    Ok(parsed_document.ast_print(
        0,
        &settings.indentation.to_string(),
        line_ending,
        settings.inline,
    ))
}

/// TODO: Temp
#[derive(Debug, Clone, thiserror::Error)]
pub enum AstParseError {
    /// Parsing a node or the document failed
    NodeParseError(#[from] NodeParseError),
    /// Error from Pest
    Pest(Box<pest::error::Error<Rule>>),
    /// The pest parser found no matching rule
    EmptyDocument,
}

impl Display for AstParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstParseError::NodeParseError(node) => write!(f, "{node}"),
            AstParseError::Pest(pest) => write!(f, "{pest}"),
            AstParseError::EmptyDocument => write!(f, "The parsed text didn't return any tokens"),
        }
    }
}

impl From<pest::error::Error<Rule>> for AstParseError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        AstParseError::Pest(Box::new(value))
    }
}

/// TODO: Temp
/// # Errors
/// TODO
pub fn parse_to_ast(text: &str) -> Result<Document, AstParseError> {
    let mut parsed_text = Grammar::parse(Rule::document, text)?;
    let document = parsed_text.next().ok_or(AstParseError::EmptyDocument)?;
    Document::try_from(document)
}

/// Documentation goes here
/// # Errors
/// TODO
pub fn pest_validate(
    text: &str,
) -> Result<pest::iterators::Pairs<Rule>, Box<pest::error::Error<Rule>>> {
    match Grammar::parse(Rule::document, text) {
        Ok(it) => Ok(it),
        Err(err) => Err(Box::new(err)),
    }
}
