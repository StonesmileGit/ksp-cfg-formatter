use crate::{
    ast_formatter::{self, Formatter},
    Indentation, LineReturn,
};

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen]
/// Can JS see this?
pub fn ksp_fmt(text: &str, insert_spaces: bool, tab_size: usize) -> String {
    let indentation = if insert_spaces {
        Indentation::Spaces(tab_size)
    } else {
        Indentation::Tabs
    };
    let formatter = Formatter::new(indentation, false, LineReturn::Identify);
    format!("{}", formatter.format_text(text))
}

#[wasm_bindgen]
/// blah
pub fn ksp_validate(text: &str) -> Option<Vec<usize>> {
    let res = ast_formatter::ast_validate(text);
    match res {
        None => None,
        Some(pos) => match pos {
            pest::error::LineColLocation::Pos(pos) => Some(vec![pos.0, pos.1]),
            pest::error::LineColLocation::Span(pos, _) => Some(vec![pos.0, pos.1]),
        },
    }
}
