use pest_derive::Parser;
// use std::fmt::Display;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;

// impl Display for Rule {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Rule::EOI => write!(f, "\"End Of File\""),
//             Rule::document => write!(f, "\"Document\""),
//             Rule::statement => write!(f, "\"Statement\""),
//             Rule::openingbracket => write!(f, "\"Opening bracket\""),
//             Rule::closingbracket => write!(f, "\"Closing bracket\""),
//             Rule::node => write!(f, "\"Node\""),
//             Rule::assignment => write!(f, "\"Key-Value pair\""),
//             Rule::identifier => write!(f, "\"Identifier\""),
//             Rule::value => write!(f, "\"Value\""),
//             Rule::Comment => write!(f, "\"Comment\""),
//             Rule::Whitespace => write!(f, "\"Whitespace\""),
//             Rule::EmptyLine => write!(f, "\"Empty Line\""),
//             Rule::Newline => write!(f, "\"Newline\""),
//             Rule::assignmentOperator => write!(f, "\"Assignment Operator\""),
//             Rule::nodeBeforeBlock => todo!(),
//             Rule::nodeBody => todo!(),
//             Rule::nameBlock => todo!(),
//             Rule::blocks => todo!(),
//             Rule::hasBranch => todo!(),
//             Rule::needsBranch => todo!(),
//             Rule::passBranch => todo!(),
//             Rule::hasBlock => todo!(),
//             Rule::needsBlock => todo!(),
//             Rule::modOrClause => write!(f, "Mod OR clause"),
//             Rule::passBlock => todo!(),
//             Rule::firstPassBlock => todo!(),
//             Rule::namedPassBlock => todo!(),
//             Rule::modName => todo!(),
//             Rule::finalPassBlock => todo!(),
//             Rule::index => todo!(),
//             Rule::arrayIndex => todo!(),
//             Rule::operator => todo!(),
//             Rule::hasBlockPart => todo!(),
//             Rule::hasNode => todo!(),
//             Rule::hasKey => todo!(),
//             Rule::hasValue => todo!(),
//             Rule::keyIdentifier => todo!(),
//             Rule::path => todo!(),
//             Rule::path_segment => todo!(),
//             Rule::hasNodeName => todo!(),
//             Rule::negation => todo!(),
//             Rule::needsMod => todo!(),
//         }
//     }
// }
