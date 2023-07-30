use crate::{
    ast_formatter::{self, Formatter},
    Indentation, LineReturn,
};

use pest::error::LineColLocation;
use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen]
/// Can JS see this?
pub fn ksp_fmt(text: &str, insert_spaces: bool, tab_size: usize) -> String {
    console_error_panic_hook::set_once();
    let indentation = if insert_spaces {
        Indentation::Spaces(tab_size)
    } else {
        Indentation::Tabs
    };
    let formatter = Formatter::new(indentation, false, LineReturn::Identify);
    format!("{}", formatter.format_text(text))
}

#[wasm_bindgen(getter_with_clone)]
/// return type for the validation
pub struct ParseError {
    /// The line where the first error occurs
    ///
    /// First line is 0
    pub line: usize,
    /// The column where the first error occurs
    ///
    /// First column is 0
    pub column: usize,
    /// The error message produced
    pub message: String,
}

#[wasm_bindgen]
/// blah
pub fn ksp_validate(text: &str) -> Option<ParseError> {
    console_error_panic_hook::set_once();
    let res = ast_formatter::ast_validate(text);
    match res {
        Ok(_) => None,
        Err(err) => Some(ParseError {
            line: match err.line_col {
                LineColLocation::Pos(pos) | LineColLocation::Span(pos, _) => pos.0 - 1,
            },
            column: match err.line_col {
                LineColLocation::Pos(pos) | LineColLocation::Span(pos, _) => pos.1 - 1,
            },
            message: err.renamed_rules(|r| r.to_string()).to_string(),
            // message: format!(
            //     "{}",
            //     match &err.variant {
            //         pest::error::ErrorVariant::ParsingError {
            //             positives,
            //             negatives,
            //         } => {
            //             match positives.len() {
            //                 0 => "".to_owned(),
            //                 1 => format!("Expected \"{}\"", positives.first().unwrap().to_string()),
            //                 x if x > 1 => format!("Expected one of \"{}\"", {
            //                     positives
            //                         .iter()
            //                         .map(|r| r.to_string())
            //                         .collect::<Vec<String>>()
            //                         .join("\", \"")
            //                 }),
            //                 _ => unreachable!(),
            //             }
            //         }
            //         _ => unreachable!(),
            //     }
            // ),
        }),
    }
}
