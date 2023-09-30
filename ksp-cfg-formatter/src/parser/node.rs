use std::ops::Deref;

use itertools::Itertools;
use nom::branch::{alt, permutation};
use nom::bytes::complete::{is_a, is_not, tag};
use nom::character::complete::{alphanumeric1, char, line_ending, space0};
use nom::combinator::{map, opt, recognize, verify};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, preceded, tuple};
use pest::iterators::Pair;

use super::nom::utils::{debug_fn, empty_line, error_till, expect, range_wrap, ws, ws_le};
use super::Ranged;
use super::{
    nom::CSTParse, ASTPrint, Comment, Error, HasBlock, Index, KeyVal, NeedsBlock, NodeItem,
    Operator, Pass, Path, Range, Rule,
};

/// A node in the config file. Both top level node and internal node
#[derive(Debug, Default, Clone)]
pub struct Node<'a> {
    top_level: bool,
    /// Optional path to node, only allowed on internal nodes
    pub path: Option<Ranged<Path<'a>>>,
    /// Optional operator
    pub operator: Option<Ranged<Operator>>,
    /// Identifier of the node
    pub identifier: Ranged<&'a str>,
    /// Optional name of the node. Same as `:HAS[name[<name>]]`
    pub name: Option<Ranged<Vec<&'a str>>>,
    /// Optional HAS block
    pub has: Option<Ranged<HasBlock<'a>>>,
    /// Optional NEEDS block
    pub needs: Option<Ranged<NeedsBlock<'a>>>,
    /// Pass for the patch to run
    pub pass: Option<Ranged<Pass<'a>>>,
    /// Optional index of the node to match
    pub index: Option<Ranged<Index>>,
    /// Optional comment after the identifier
    pub id_comment: Option<Ranged<Comment<'a>>>,
    /// optional comments between identifier line and opening bracket
    pub comments_after_newline: Vec<Ranged<Comment<'a>>>,
    /// Items inside the node
    pub block: Vec<NodeItem<'a>>,
    /// Optional trailing comment after the closing bracket
    pub trailing_comment: Option<Ranged<Comment<'a>>>,
}

impl<'a> Node<'a> {
    /// Indicates if a node is a top-level node or not
    #[must_use]
    pub const fn top_level(&self) -> bool {
        self.top_level
    }
    /// Returns an iterator over all of the Nodes contained within this node
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Ranged<Node>> {
        self.block.iter().filter_map(|n| {
            if let NodeItem::Node(node) = n {
                Some(node)
            } else {
                None
            }
        })
    }
    /// Returns an iterator over all of the Assignments contained within this node
    pub fn iter_keyvals(&self) -> impl Iterator<Item = &Ranged<KeyVal>> {
        self.block.iter().filter_map(|n| {
            if let NodeItem::KeyVal(kv) = n {
                Some(kv)
            } else {
                None
            }
        })
    }
}

pub(crate) fn parse_block_items(pair: Pair<Rule>, top_level: bool) -> Result<Vec<NodeItem>, Error> {
    assert!(matches!(pair.as_rule(), Rule::nodeBody | Rule::document));
    let mut block_items = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::node => block_items.push(Ok(NodeItem::Node(Ranged::<Node>::try_from((
                pair, top_level,
            ))?))),
            Rule::Comment => block_items.push(Ok(NodeItem::Comment(
                Ranged::<Comment>::try_from(pair).expect("Parsing a comment is Infallable"),
            ))),
            Rule::assignment => {
                block_items.push(Ok(NodeItem::KeyVal(Ranged::<KeyVal>::try_from((
                    pair, top_level,
                ))?)));
            }
            Rule::EmptyLine => block_items.push(Ok(NodeItem::EmptyLine)),
            Rule::EOI | Rule::Newline => (),
            _ => unreachable!(),
        }
    }
    block_items.into_iter().collect()
}

impl<'a> TryFrom<(Pair<'a, Rule>, bool)> for Ranged<Node<'a>> {
    type Error = Error;

    fn try_from(rule_bool: (Pair<'a, Rule>, bool)) -> Result<Self, Self::Error> {
        let rule = rule_bool.0;
        let range = Range::from(&rule);
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
                        node.trailing_comment = Some(
                            Ranged::<Comment>::try_from(pair)
                                .expect("Parsing a comment is Infallable"),
                        );
                    } else if newline_seen {
                        node.comments_after_newline.push(
                            Ranged::<Comment>::try_from(pair)
                                .expect("Parsing a comment is Infallable"),
                        );
                    } else {
                        node.id_comment = Some(
                            Ranged::<Comment>::try_from(pair)
                                .expect("Parsing a comment is Infallable"),
                        );
                    }
                }
                Rule::openingbracket | Rule::closingbracket => (),
                Rule::Newline => newline_seen = true,

                Rule::identifier => node.identifier = Ranged::new(pair.as_str(), pair.into()),
                Rule::nameBlock => {
                    let names: Vec<&'a str> = pair.as_str().split('|').collect();
                    node.name = Some(Ranged::new(names, pair.into()));
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
                    node.has = Some(Ranged::<HasBlock>::try_from(pair)?);
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
                    node.needs = Some(Ranged::<NeedsBlock>::try_from(pair)?);
                }
                Rule::passBlock => {
                    if node.pass.is_some() {
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
                    node.pass = Some(Ranged::<Pass>::try_from(pair).expect("Should be Infallable"));
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
                    node.index = Some(Ranged::<Index>::try_from(pair)?);
                }
                Rule::operator => {
                    let op = Some(Ranged::<Operator>::try_from(pair.clone())?);
                    if let Some(op) = &op {
                        if matches!(op.deref(), Operator::Rename) {
                            return Err(Error {
                                reason: crate::parser::Reason::Custom(
                                    "Found rename operator on node".to_string(),
                                ),
                                source_text: pair.as_str().to_string(),
                                location: Some(pair.into()),
                            });
                        }
                    }
                    node.operator = op;
                }
                Rule::path => {
                    node.path = Some(
                        Ranged::<Path>::try_from(pair)
                            .expect("Parsing path is supposedly Infallable"),
                    );
                }
                Rule::nodeBody => {
                    node.block = parse_block_items(pair, false)?;
                    body_seen = true;
                }
                _ => unreachable!(),
            }
        }
        Ok(Ranged::new(node, range))
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
            self.pass.clone().map_or(String::new(), |p| p.to_string()),
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index
                .as_deref()
                .map_or(String::new(), |i| i.to_string()),
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
    len += arg
        .pass
        .clone()
        .map_or(0, |p| p.to_string().chars().count());
    len += arg
        .index
        .as_deref()
        .map_or(0, |id| id.to_string().chars().count());

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

use super::nom::{utils::ignore_line_ending, IResult, LocatedSpan};

impl<'a> CSTParse<'a, Ranged<Node<'a>>> for Node<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Node<'a>>> {
        let top_level = input.extra.state.top_level.clone();
        let path = opt(preceded(tag("#"), Path::parse));
        let operator = opt(Operator::parse);
        // identifier = ${ ("-" | "_" | "." | "+" | "*" | "?" | LETTER | ASCII_DIGIT)+ }
        let identifier = range_wrap(map(
            recognize(many1(alt((
                alphanumeric1::<LocatedSpan, _>,
                is_a("-_.+*?"),
            )))),
            |inner| *inner.fragment(),
        ));
        let name = opt(parse_name);
        let has = opt(HasBlock::parse);
        let needs = opt(NeedsBlock::parse);
        let pass = opt(Pass::parse);
        let index = opt(Index::parse);
        let id_comment = opt(Comment::parse);
        let comments_after_newline = many0(Comment::parse);
        let block = preceded(opt(line_ending), preceded(space0, parse_block));
        let trailing_comment = opt(Comment::parse);
        let node = tuple((
            debug_fn(path, "Got path", true),
            debug_fn(operator, "Got operator", true),
            debug_fn(identifier, "Got node id", true),
            debug_fn(expect(name, "Expected name"), "Got name", true),
            // FIXME: Order of has/needs/pass matters at the moment. Rework to take them in any order (one time, or many, for error handling?)
            many0(verify(
                permutation((
                    debug_fn(has, "Got has", true),
                    debug_fn(pass, "Got pass", true),
                    debug_fn(needs, "Got needs", true),
                )),
                |a| a.0.is_some() | a.1.is_some() | a.2.is_some(),
            )),
            debug_fn(expect(index, "Expected index"), "Got index", true),
            debug_fn(
                expect(id_comment, "Expected id_comment"),
                "Got id_comment",
                true,
            ),
            debug_fn(
                expect(comments_after_newline, "Expected comments after newline"),
                "Got comments after newline",
                true,
            ),
            debug_fn(block, "Got block", true),
            debug_fn(
                expect(trailing_comment, "Expected trailing comment"),
                "Got trailing comment",
                true,
            ),
        ));
        let res = range_wrap(map(ws(node), |inner| Node {
            top_level,
            path: inner.0,
            operator: inner.1,
            // identifier: inner.2.map_or("ERROR", |s| s.fragment()),
            identifier: inner.2,
            name: inner.3.unwrap_or(None),
            has: inner
                .4
                .iter()
                .filter_map(|a| a.0.clone())
                .collect_vec()
                .first()
                .cloned(),
            needs: inner
                .4
                .iter()
                .filter_map(|a| a.2.clone())
                .collect_vec()
                .first()
                .cloned(),
            pass: inner
                .4
                .into_iter()
                .filter_map(|a| a.1)
                .collect_vec()
                .first()
                .cloned(),
            index: inner.5.unwrap_or(None),
            id_comment: inner.6.unwrap_or(None),
            comments_after_newline: inner.7.unwrap_or_default(),
            // block: inner.10.unwrap_or(vec![]),
            block: inner.8,
            trailing_comment: inner.9.unwrap_or(None),
        }))(input);
        // extra line to handle life time stuff
        res
    }
}

fn parse_name(input: LocatedSpan) -> IResult<Ranged<Vec<&str>>> {
    let list = delimited(
        tag("["),
        separated_list1(tag("|"), is_not("|]")),
        expect(tag("]"), "Expected closing ]"),
    );
    range_wrap(map(list, |inner| {
        inner.iter().map(|e: &LocatedSpan| *e.fragment()).collect()
    }))(input)
}

/// Takes a parser and sets the settings according to what is needed for parsing an inner block, and then setting them back as needed on the returned settings as needed
pub(crate) fn settings_for_inner_block<'a, F, T>(
    mut parser: F,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    move |input: LocatedSpan<'a>| {
        let top_level = input.extra.state.top_level.clone();
        let mut input = input;
        input.extra.state.top_level = false;
        let res = parser(input);
        match res {
            Ok(mut it) => {
                it.0.extra.state.top_level = top_level;
                Ok(it)
            }
            Err(nom::Err::Error(mut err) | nom::Err::Failure(mut err)) => {
                err.input.extra.state.top_level = top_level;
                Err(nom::Err::Error(err))
            }
            Err(err) => Err(err),
        }
    }
}

fn parse_block<'a>(input: LocatedSpan<'a>) -> IResult<Vec<NodeItem>> {
    let block = delimited(
        char('{'),
        ws_le(many0(ws(alt((
            map(ignore_line_ending(ws(Comment::parse)), |c| {
                NodeItem::Comment(c)
            }),
            map(ws(empty_line), |_| NodeItem::EmptyLine),
            map(ws(KeyVal::parse), NodeItem::KeyVal),
            settings_for_inner_block(map(ignore_line_ending(ws(Node::parse)), NodeItem::Node)),
            debug_fn(
                map(
                    recognize(error_till(verify(is_not("}\r\n"), |s: &LocatedSpan| {
                        s.len() > 0
                    }))),
                    |s| NodeItem::Error(Ranged::new(s.clone().fragment(), s.into())),
                ),
                "Got an error while parsing node. Skipped line",
                true,
            ),
        ))))),
        expect(char('}'), "Expected closing }"),
    );
    map(block, |inner: Vec<NodeItem>| inner)(input)
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_node() {
        let input = "node { key = val }\r\n";
        let res = Node::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_node_2() {
        let input = "node\r\n{\r\n\tkey = val\r\n\tkey = val\r\n}\r\n";
        let res = Node::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{:#?}", err),
        }
    }
}
