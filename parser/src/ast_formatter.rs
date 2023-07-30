mod printer;
mod reader;

use self::{printer::Document, reader::parse_block_items};
use pest::Parser;
use printer::ASTPrint;
use reader::{Grammar, Rule};
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

use super::{Indentation, LineReturn};

/// Struct for holding the settings to use for formatting. use `self.format_text()` to format text
///
/// Example:
/// ```
/// use ksp_cfg_formatter_lib::{ast_formatter::Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Tabs;
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false, line_return);
/// # // this is needed to test the code, but not important to readers
/// # let input = "".to_owned();
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
    /// use ksp_cfg_formatter_lib::{ast_formatter::Formatter, Indentation, LineReturn};
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
    /// use ksp_cfg_formatter_lib::{ast_formatter::Formatter, Indentation, LineReturn};
    ///
    /// let indentation = Indentation::Tabs;
    /// let line_return = LineReturn::Identify;
    /// let formatter = Formatter::new(indentation, false, line_return);
    /// # // this is needed to test the code, but not important to readers
    /// # let input = "".to_owned();
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
                statements: parse_block_items(document.into_inner()),
            };
            let line_ending = if use_crlf { "\r\n" } else { "\n" };
            return parsed_document.ast_print(
                0,
                &settings.indentation.to_string(),
                line_ending,
                settings.inline,
            );
        }
        Err(err) => {
            dbg!("{}", &text);
            dbg!(&document_res);
            panic!("{}", err);
        }
    };
}

/// Documentation goes here
pub fn ast_validate(text: &str) -> Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> {
    Grammar::parse(Rule::document, text)
}
