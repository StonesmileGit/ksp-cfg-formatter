use pest::iterators::Pair;

use crate::{reader::Rule, AstParseError};

use super::{node::parse_block_items, node_item::NodeItem, ASTPrint};

#[derive(Debug)]
pub struct Document<'a> {
    pub statements: Vec<NodeItem<'a>>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Document<'a> {
    type Error = AstParseError<'a>;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let statements = parse_block_items(rule);
        match statements {
            Ok(statements) => Ok(Document { statements }),
            Err(err) => Err(AstParseError::NodeParseError(err)),
        }
    }
}

impl<'a> ASTPrint for Document<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for item in &self.statements {
            output.push_str(&item.ast_print(depth, indentation, line_ending, should_collapse));
        }
        output
    }
}
