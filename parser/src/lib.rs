//! The crate currently provides two formatters; One based on iterating over tokens, and one based on chars
#![warn(missing_docs)]
/// Contains code to interface with TypeScript
#[cfg(target_family = "wasm")]
pub mod wasm_bindings;

mod printer;
mod reader;

use self::printer::Document;
use pest::Parser;
use printer::{node::parse_block_items, ASTPrint};
use reader::{Grammar, Rule};
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

/// Defines which End of Line sequence to be used
///
/// Can have the values `LF`, `CRLF` or `Identify`.
///
/// When using `Identify`, the formatter tries to figure out what sequence to use, based on the provided text.
///
/// Example:
/// ```
/// use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
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
/// use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
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
/// use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
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
    /// use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
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
    /// TODO: Explain the parts of the formatter.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
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
        #[cfg(not(target_family = "wasm"))]
        let total = Instant::now();

        let text = ast_format(text, self);

        #[cfg(not(target_family = "wasm"))]
        let total_time = total.elapsed();

        #[cfg(not(target_family = "wasm"))]
        if false {
            println!("{total_time:?} Total");
        }
        text
    }
}

fn ast_format(text: &str, settings: &Formatter) -> String {
    let use_crlf = if matches!(settings.line_return, LineReturn::Identify) {
        text.contains("\r\n")
    } else {
        matches!(settings.line_return, LineReturn::CRLF)
    };
    let document_res = Grammar::parse(Rule::document, text);
    match &document_res {
        Ok(res) => {
            let document = res.clone().next().unwrap();
            // dbg!(&document);
            let parsed_document = Document {
                statements: parse_block_items(document).ok().unwrap(),
            };
            let line_ending = if use_crlf { "\r\n" } else { "\n" };
            parsed_document.ast_print(
                0,
                &settings.indentation.to_string(),
                line_ending,
                settings.inline,
            )
        }
        Err(err) => {
            dbg!("{}", &text);
            dbg!(&document_res);
            panic!("{}", err);
        }
    }
}

/// TODO: Temp
pub enum AstParseError {
    /// Temp variant before error is further developed
    Temp,
    /// Error from Pest
    Pest(Box<pest::error::Error<Rule>>),
}

/// TODO: Temp
/// # Errors
/// TODO
pub fn parse_to_ast(text: &str) -> Result<Document, AstParseError> {
    let parsed_text = Grammar::parse(Rule::document, text);
    match parsed_text {
        Ok(doc) => {
            let document = doc
                .clone()
                .next()
                .expect("The parsed text has to contain a Document node");
            let statements = parse_block_items(document);
            match statements {
                Ok(statements) => Ok(Document { statements }),
                Err(_) => Err(AstParseError::Temp),
            }
        }
        Err(err) => Err(AstParseError::Pest(Box::new(err))),
    }
}

/// Documentation goes here
/// # Errors
/// TODO
pub fn ast_validate(
    text: &str,
) -> Result<pest::iterators::Pairs<Rule>, Box<pest::error::Error<Rule>>> {
    match Grammar::parse(Rule::document, text) {
        Ok(it) => Ok(it),
        Err(err) => Err(Box::new(err)),
    }
}
