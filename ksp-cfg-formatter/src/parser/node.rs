use itertools::Itertools;
use pest::iterators::Pair;

use super::{
    ASTPrint, Comment, Error, HasBlock, Index, KeyVal, NeedsBlock, NodeItem, Operator, Pass, Path,
    Rule,
};

/// A node in the config file. Both top level node and internal node
#[derive(Debug, Default)]
pub struct Node<'a> {
    top_level: bool,
    /// Optional path to node, only allowed on internal nodes
    pub path: Option<Path<'a>>,
    /// Optional operator
    pub operator: Option<Operator>,
    /// Identifier of the node
    pub identifier: &'a str,
    /// Optional name of the node. Same as `:HAS[name[<name>]]`
    pub name: Option<Vec<&'a str>>,
    /// Optional HAS block
    pub has: Option<HasBlock<'a>>,
    /// Optional NEEDS block
    pub needs: Option<NeedsBlock<'a>>,
    /// Pass for the patch to run
    pub pass: Pass<'a>,
    /// Optional index of the node to match
    pub index: Option<Index>,
    /// Optional comment after the identifier
    pub id_comment: Option<Comment<'a>>,
    /// optional comments between identifier line and opening bracket
    pub comments_after_newline: Vec<Comment<'a>>,
    /// Items inside the node
    pub block: Vec<NodeItem<'a>>,
    /// Optional trailing comment after the closing bracket
    pub trailing_comment: Option<Comment<'a>>,
}

impl<'a> Node<'a> {
    /// Indicates if a node is a top-level node or not
    pub fn top_level(&self) -> bool {
        self.top_level
    }
}

pub(crate) fn parse_block_items(pair: Pair<Rule>, top_level: bool) -> Result<Vec<NodeItem>, Error> {
    assert!(matches!(pair.as_rule(), Rule::nodeBody | Rule::document));
    let mut block_items = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::node => block_items.push(Ok(NodeItem::Node(Node::try_from((pair, top_level))?))),
            Rule::Comment => block_items.push(Ok(NodeItem::Comment(
                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
            ))),
            Rule::assignment => {
                block_items.push(Ok(NodeItem::KeyVal(KeyVal::try_from((pair, top_level))?)))
            }
            Rule::EmptyLine => block_items.push(Ok(NodeItem::EmptyLine)),
            Rule::EOI | Rule::Newline => (),
            _ => unreachable!(),
        }
    }
    block_items.into_iter().collect()
}

impl<'a> TryFrom<(Pair<'a, Rule>, bool)> for Node<'a> {
    type Error = Error;

    fn try_from(rule_bool: (Pair<'a, Rule>, bool)) -> Result<Self, Self::Error> {
        let rule = rule_bool.0;
        assert!(matches!(rule.as_rule(), Rule::node));

        let mut node = Node {
            top_level: rule_bool.1,
            ..Default::default()
        };

        let mut body_seen = false;
        let mut newline_seen = false;

        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::Comment => {
                    if body_seen {
                        node.trailing_comment =
                            Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                    } else if newline_seen {
                        node.comments_after_newline.push(
                            Comment::try_from(pair).expect("Parsing a comment is Infallable"),
                        );
                    } else {
                        node.id_comment =
                            Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                    }
                }
                Rule::openingbracket | Rule::closingbracket => (),
                Rule::Newline => newline_seen = true,

                Rule::identifier => node.identifier = pair.as_str(),
                Rule::nameBlock => {
                    let names: Vec<&'a str> = pair.as_str().split('|').collect();
                    if names.len() > 1 && !node.top_level {
                        // TODO: This is technically a warning, and not an error
                        return Err(Error {
                            reason: crate::parser::Reason::Custom("Node name-filter contained multiple items, but is not top level node".to_string()),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.name = Some(names);
                }
                Rule::hasBlock => {
                    if node.has.is_some() {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Only one 'HAS' block is allowed".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.has = Some(HasBlock::try_from(pair)?);
                }
                Rule::needsBlock => {
                    if node.needs.is_some() {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Only one 'NEEDS' block is allowed".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.needs = Some(NeedsBlock::try_from(pair)?);
                }
                Rule::passBlock => {
                    if node.pass != Pass::Default {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Only one pass is allowed".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    if !node.top_level {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Pass specifiers are only allowed on top-level nodes".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.pass = Pass::try_from(pair).expect("Should be Infallable");
                }
                Rule::index => {
                    if node.index.is_some() {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Only one 'INDEX' block is allowed".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.index = Some(super::indices::Index::try_from(pair)?);
                }
                Rule::operator => {
                    let op = Some(Operator::try_from(pair.clone())?);
                    if matches!(op, Some(Operator::Rename)) {
                        return Err(Error {
                            reason: crate::parser::Reason::Custom(
                                "Found rename operator on node".to_string(),
                            ),
                            source_text: pair.as_str().to_string(),
                            location: Some(pair.into()),
                        });
                    }
                    node.operator = op;
                }
                Rule::path => {
                    node.path =
                        Some(Path::try_from(pair).expect("Parsing path is supposedly Infallable"));
                }
                Rule::nodeBody => {
                    node.block = parse_block_items(pair, false)?;
                    body_seen = true;
                }
                _ => unreachable!(),
            }
        }
        Ok(node)
    }
}

// TODO: Assignments are performed before nodes, so order them that way (move assignments before any nodes)
// Thoughts:
//      What about comments and newlines?
//      This should be in a separate part of the parser, a sort of middle tool that is run after parsing, before printing
impl<'a> ASTPrint for Node<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for comment in &self.comments_after_newline {
            output.push_str(
                comment
                    .ast_print(depth, indentation, line_ending, should_collapse)
                    .as_str(),
            );
        }
        let indentation_str = indentation.repeat(depth);
        let complete_node_name = format!(
            "{}{}{}{}{}{}{}{}{}",
            if self.path.is_some() { "#" } else { "" },
            self.path.clone().map_or(String::new(), |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.identifier,
            self.name.clone().map_or(String::new(), |name| format!(
                "[{}]",
                name.iter().format("|")
            )),
            self.has.clone().unwrap_or_default(),
            self.pass,
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index.map_or(String::new(), |i| i.to_string()),
        );
        output.push_str(
            match self.block.len() {
                0 if self.id_comment.is_none() => {
                    format!(
                        "{}{} {{}}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                        line_ending
                    )
                }
                1 if should_collapse && short_node(self) => {
                    format!(
                        "{}{} {{ {} }}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.block
                            .first()
                            .unwrap()
                            .ast_print(0, indentation, "", should_collapse),
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                        line_ending
                    )
                }
                _ => {
                    let mut output = format!(
                        "{}{}{}{}{}{{{}",
                        indentation_str,
                        complete_node_name,
                        self.id_comment.as_ref().map_or_else(|| "", |c| c.text),
                        line_ending,
                        indentation_str,
                        line_ending
                    );
                    for statement in &self.block {
                        output.push_str(
                            statement
                                .ast_print(depth + 1, indentation, line_ending, should_collapse)
                                .as_str(),
                        );
                    }
                    output.push_str(&indentation_str);
                    output.push('}');
                    output.push_str(
                        self.trailing_comment
                            .as_ref()
                            .map_or_else(|| "", |c| c.text),
                    );
                    output.push_str(line_ending);
                    output
                }
            }
            .as_str(),
        );
        output
    }
}

fn short_node(arg: &Node) -> bool {
    const MAX_LENGTH: usize = 72;
    if arg.id_comment.is_some() {
        return false;
    }
    let mut len = 7; // Include the opening/closing bracket and spaces around operator
    len += arg
        .path
        .clone()
        .map_or(0, |path| path.to_string().chars().count());
    len += arg
        .operator
        .clone()
        .map_or(0, |op| op.to_string().chars().count());
    len += arg.identifier.chars().count();
    len += arg
        .name
        .clone()
        .map_or(0, |name| name.iter().map(|e| e.chars().count()).sum());
    len += arg
        .has
        .clone()
        .map_or(0, |has| has.to_string().chars().count());
    len += arg
        .needs
        .clone()
        .map_or(0, |needs| needs.to_string().chars().count());
    len += arg.pass.clone().to_string().chars().count();
    len += arg.index.map_or(0, |id| id.to_string().chars().count());

    match arg.block.first().unwrap() {
        NodeItem::KeyVal(kv) => {
            if kv.operator.is_some() {
                len += 1;
            }
            len += kv.key.chars().count();
            len += kv.assignment_operator.to_string().chars().count();
            len += kv.val.chars().count();
            if kv.comment.is_some() {
                return false;
            };
        }
        _ => return false,
    }
    len <= MAX_LENGTH
}
