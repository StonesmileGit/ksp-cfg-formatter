pub mod assignment_operator;
pub mod comment;
pub mod document;
pub mod has;
pub mod indices;
pub mod key_val;
pub mod needs;
pub mod node;
pub mod node_item;
pub mod operator;
pub mod pass;
pub mod path;

pub trait ASTPrint {
    #[must_use]
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String;
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;
