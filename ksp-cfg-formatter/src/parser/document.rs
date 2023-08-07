use pest::iterators::Pair;

use super::{node::parse_block_items, node_item::NodeItem, ASTPrint, Error, Rule};

/// Contains all the statements of a file
#[derive(Debug)]
pub struct Document<'a> {
    /// List of all the statements. Can be `Node`s, `Comment`s, or `EmptyLine`s
    pub statements: Vec<NodeItem<'a>>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Document<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Error> {
        for statement in rule.clone().into_inner() {
            if statement.as_rule() == Rule::assignment {
                return Err(Error {
                    reason: super::Reason::Custom("Top level assignment found".to_string()),
                    source_text: statement.as_str().to_string(),
                    location: Some(statement.into()),
                });
            }
        }
        let statements = parse_block_items(rule, true)?;
        Ok(Document { statements })
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
