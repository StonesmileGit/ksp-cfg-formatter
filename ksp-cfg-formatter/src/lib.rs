//! Parser and formatter for Kerbal Space Program config files, including Module Manager syntax
#![warn(missing_docs)]
/// Contains code to interface with TypeScript
// #[cfg(target_family = "wasm")]
// pub mod wasm_bindings;

/// Contains the types of the parser
pub mod parser;
/// Functions to perform transformations on the parsed AST
pub mod transformer;

/// Contains methods to lint the generated AST
pub mod linter;

use linter::Diagnostic;
use log::warn;
use parser::{parse, ASTPrint, Document};

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
/// let formatter = Formatter::new(indentation, Some(false), line_return);
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
/// let formatter = Formatter::new(indentation, Some(false), line_return);
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
/// let formatter = Formatter::new(indentation, Some(false), line_return);
/// # // this is needed to test the code, but not important to readers
/// # let input = String::new();
/// let output = formatter.format_text(&input);
/// ```
///
/// See [`Formatter::format_text()`]
pub struct Formatter {
    indentation: Indentation,
    inline: Option<bool>,
    line_return: LineReturn,
    fail_silent: bool,
}

impl Formatter {
    /// Constructs a new `Formatter` with the settings provided.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let formatter = Formatter::new(Indentation::Tabs, Some(false), LineReturn::Identify);
    /// ```
    #[must_use]
    pub const fn new(
        indentation: Indentation,
        inline: Option<bool>,
        line_return: LineReturn,
    ) -> Self {
        Self {
            indentation,
            inline,
            line_return,
            fail_silent: false,
        }
    }

    /// Makes the parser fail silently, returning the original text instead of causing a Panic
    #[must_use]
    pub const fn fail_silent(self) -> Self {
        Self {
            indentation: self.indentation,
            inline: self.inline,
            line_return: self.line_return,
            fail_silent: true,
        }
    }

    /// Takes the provided text and formats it according to the settings of the `Formatter`
    ///
    /// If the formatter is set to fail silently, and formatting fails, the orginal text is returned unchanged
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let indentation = Indentation::Tabs;
    /// let line_return = LineReturn::Identify;
    /// let formatter = Formatter::new(indentation, Some(false), line_return);
    /// # // this is needed to test the code, but not important to readers
    /// # let input = String::new();
    /// let output = formatter.format_text(&input);
    /// ```
    /// # Errors
    /// If formatter is set to fail silently, the original text is returned. Otherwise the errors are returned
    pub fn format_text(&self, text: &str) -> Result<String, Vec<parser::Error>> {
        match ast_format(text, self) {
            Ok(res) => Ok(res),
            Err(err) => {
                if self.fail_silent {
                    Ok(text.to_string())
                } else {
                    Err(err)
                }
            }
        }
    }
}

fn ast_format(text: &str, settings: &Formatter) -> Result<String, Vec<parser::Error>> {
    let use_crlf = if matches!(settings.line_return, LineReturn::Identify) {
        text.contains("\r\n")
    } else {
        matches!(settings.line_return, LineReturn::CRLF)
    };
    let (parsed_document, errors) = parse(text);
    for error in &errors {
        warn!("{error:#?}");
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    // let parsed_document = transformer::assignments_first(parsed_document)?;
    // let parsed_document = transformer::assignment_padding(parsed_document);
    let line_ending = if use_crlf { "\r\n" } else { "\n" };
    Ok(parsed_document.ast_print(
        0,
        &settings.indentation.to_string(),
        line_ending,
        settings.inline,
    ))
}

/// Parses the text to a `Document` struct
/// # Errors
/// If any part of the parser fails, the returned error indicates what caused it, where it occured, and the source text for the error
pub fn parse_to_ast(text: &str) -> Result<Document, (Vec<parser::Error>, Vec<Diagnostic>)> {
    let (parsed_document, errors) = parse(text);
    let diagnostics = linter::lint_ast(&parsed_document, None);
    if !errors.is_empty() || !diagnostics.is_empty() {
        return Err((errors, diagnostics));
    }
    Ok(parsed_document)
}
