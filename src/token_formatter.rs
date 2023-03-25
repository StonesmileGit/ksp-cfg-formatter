use std::{
    collections::{linked_list::CursorMut, LinkedList},
    fmt::Display,
};

use logos::Logos;
use std::time::Instant;

/// Defines which End of Line sequence to be used
///
/// TODO: Explain in detail
///
/// Example:
/// ```
/// use ksp_cfg_formatter::token_formatter::{Formatter, Indentation, LineReturn};
///
/// let line_return = LineReturn::LF;
///
/// let indentation = Indentation::Tabs;
/// let formatter = Formatter::new(indentation, false);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms, dead_code)]
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
/// use ksp_cfg_formatter::token_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Spaces(4);
///
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false);
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

impl From<usize> for Indentation {
    // TODO: This uses sentinel values, change code
    fn from(n: usize) -> Self {
        if n == 0 {
            Self::Tabs
        } else {
            Self::Spaces(n)
        }
    }
}

/// Struct for holding the settings to use for formatting. use `self.format_text()` to format text
///
/// Example:
/// ```
/// use ksp_cfg_formatter::token_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Tabs;
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false);
/// # // this is needed to test the code, but not important to readers
/// # let input = "".to_owned();
/// let output = formatter.format_text(&input);
/// ```
///
/// See [`Formatter::format_text()`]
pub struct Formatter {
    indentation: Indentation,
    inline: bool,
}

impl Formatter {
    /// Constructs a new `Formatter` with the settings provided.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::token_formatter::{Formatter, Indentation};
    ///
    /// let formatter = Formatter::new(Indentation::Tabs, false);
    /// ```
    pub const fn new(indentation: Indentation, inline: bool) -> Self {
        Self {
            indentation,
            inline,
        }
    }

    // Formats the provided text according to the settings of the Formatter
    /// Takes the provided text and formats it according to the settings of the `Formatter`
    ///
    /// TODO: Explain the parts of the formatter.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::token_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let indentation = Indentation::Tabs;
    /// let line_return = LineReturn::Identify;
    /// let formatter = Formatter::new(indentation, false);
    /// # // this is needed to test the code, but not important to readers
    /// # let input = "".to_owned();
    /// let output = formatter.format_text(&input);
    /// ```
    pub fn format_text(&self, text: &str) -> String {
        let debug_print = false;
        let total = Instant::now();

        let start = Instant::now();
        let mut token_list = Token::lexer(text).collect::<LinkedList<Token>>();
        let tokenize_time = start.elapsed();

        // dbg!(&token_list);

        let a = check_brackets(&token_list);
        if !a {
            return text.to_owned();
        }

        let formatting_timer = Instant::now();

        let start = Instant::now();
        remove_leading_whitespace(&mut token_list.cursor_front_mut());
        let leading_whitespace_time = start.elapsed();

        let start = Instant::now();
        format_blocks(&mut token_list.cursor_front_mut(), &true, &self.inline);
        let format_blocks_time = start.elapsed();

        let start = Instant::now();
        indentation(&mut token_list.cursor_front_mut(), self.indentation);
        let indentation_time = start.elapsed();

        let start = Instant::now();
        remove_trailing_whitespace(&mut token_list.cursor_front_mut());
        let trailing_whitespace_time = start.elapsed();

        let start = Instant::now();
        remove_leading_and_trailing_newlines(&mut token_list.cursor_front_mut());
        let newlines_time = start.elapsed();

        let formatting_time = formatting_timer.elapsed();

        let start = Instant::now();
        let mut text = String::new();
        for token in token_list {
            text.push_str(&token.to_string());
        }
        let to_string_time = start.elapsed();

        let total_time = total.elapsed();

        if debug_print {
            println!("{tokenize_time:?} Tokenizing input");
            println!("{leading_whitespace_time:?} Removed leading whitespace");
            println!("{format_blocks_time:?} Formatted blocks");
            println!("{indentation_time:?} Fixed indentation");
            println!("{trailing_whitespace_time:?} Removed trailing whitespace");
            println!("{newlines_time:?} Trailing whitelines");
            println!("{to_string_time:?} Converting to string");
            println!("{formatting_time:?} Total formatting");
            println!("{total_time:?} Total");
        }
        text
    }
}

/// This enum represents tokens
#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    /// Represents a comment. Includes the leading `//`
    #[regex(r"//.*")]
    Comment(&'a str),

    /// Token representing a newline
    #[regex(r"\r?\n")]
    NewLine,

    /// Token representing an opening bracket, `{`
    #[token(r"{")]
    OpeningBracket,

    /// Token representing a closing bracket, `}`
    #[token(r"}")]
    ClosingBracket,

    /// Token representing any whitespace Currently doesn't care about length
    #[regex(r"[ \t]+")]
    Whitespace(&'a str),

    /// Token representing =
    #[token(r"=")]
    Equals,

    /// Token representing any text
    #[regex(r"[^{} \t\r\n]+")]
    Text(&'a str),

    /// Token representing any error
    #[error]
    Error,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Comment(comment) => write!(f, "{comment}"),
            Token::OpeningBracket => write!(f, "{{"),
            Token::ClosingBracket => write!(f, "}}"),
            Token::Whitespace(space) => write!(f, "{space}"),
            Token::Text(text) => write!(f, "{text}"),
            Token::Error => todo!(),
            Token::NewLine => writeln!(f),
            Token::Equals => write!(f, "="),
        }
    }
}

fn check_brackets(list: &LinkedList<Token>) -> bool {
    let mut indent = 0;
    for item in list {
        if *item == Token::OpeningBracket {
            indent += 1;
        }
        if *item == Token::ClosingBracket {
            indent -= 1;
        }
        if indent < 0 {
            return false;
        }
    }
    indent == 0
}

/// Removes leading whitespace
pub fn remove_leading_whitespace(cursor: &mut CursorMut<Token>) {
    while let Some(token) = cursor.current() {
        if matches!(token, Token::NewLine) {
            if let Some(Token::Whitespace(_)) = cursor.peek_next() {
                cursor.move_next();
                cursor.remove_current();
                cursor.move_prev();
                cursor.move_prev();
            }
        }
        cursor.move_next();
    }
}

/// Removes trailing whitespace
pub fn remove_trailing_whitespace(cursor: &mut CursorMut<Token>) {
    while let Some(token) = cursor.current() {
        if matches!(token, Token::NewLine) {
            if let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                cursor.move_prev();
                cursor.remove_current();
                cursor.move_prev();
            }
        }
        cursor.move_next();
    }
}

/// Remove extra newlines at start/end of file
pub fn remove_leading_and_trailing_newlines(cursor: &mut CursorMut<Token>) {
    while cursor.current() == Some(&mut Token::NewLine) {
        cursor.pop_front();
    }
    cursor.move_prev();
    while cursor.peek_prev() == Some(&mut Token::NewLine) {
        cursor.pop_back();
    }
    // TODO: Change to front if newline appears at front of list
    cursor.push_back(Token::NewLine);
}

/// Generates correct indentation
pub fn indentation(cursor: &mut CursorMut<Token>, indentation: Indentation) {
    let indent_text = match indentation {
        // TODO: use n
        Indentation::Spaces(_n) => Token::Whitespace("    "),
        Indentation::Tabs => Token::Whitespace("\t"),
    };
    let mut level: usize = 0;
    while let Some(token) = cursor.current() {
        match token {
            Token::NewLine => {
                if cursor.peek_next() == Some(&mut Token::ClosingBracket) {
                    debug_assert!(level > 0);
                    level -= 1;
                }
                // if cursor.peek_next().is_some() {
                //     cursor.move_next();
                // }
                for _ in 0..level {
                    // TODO: CHange to create one token per line, instead of multiple ones of the base size. That way insert_after isn't a problem
                    cursor.insert_after(indent_text);
                    cursor.move_next();
                }
            }
            Token::OpeningBracket => {
                level += 1;
                if cursor.peek_next() == Some(&mut Token::ClosingBracket) {
                    debug_assert!(level > 0);
                    level -= 1;
                }
            }
            _ => {
                if cursor.peek_next() == Some(&mut Token::ClosingBracket) {
                    debug_assert!(level > 0);
                    level -= 1;
                }
            }
        }
        cursor.move_next();
    }
}

/// blah
pub fn format_blocks(cursor: &mut CursorMut<Token>, top_level: &bool, inline: &bool) {
    // println!("cursor pos: {}", cursor.index().unwrap());
    if !top_level {
        while let Some(token) = cursor.current() {
            match token {
                Token::OpeningBracket => {
                    cursor.move_next();
                    break;
                }
                _ => cursor.move_next(),
            }
        }
    }
    while let Some(token) = cursor.current() {
        match token {
            Token::OpeningBracket => {
                // println!("found block");
                // We have reached the beginning of a new block. Go back and find the identifier, split the list and start recursion
                // TODO: go back and find identifier
                while let Some(token) = cursor.current() {
                    match token {
                        Token::Text(_) => break,
                        _ => cursor.move_prev(),
                    }
                }
                let first = cursor.split_before();
                format_blocks(cursor, &false, inline);
                debug_assert!(
                    matches!(cursor.current(), Some(Token::ClosingBracket)),
                    "token was {:?}",
                    cursor,
                );
                // cursor is now after the block. We want to prepend the "first" list to the cursor.
                let mut len: usize = 0;
                while cursor.peek_prev().is_some() {
                    cursor.move_prev();
                    len += 1;
                }
                cursor.splice_before(first);
                for _ in 0..len {
                    cursor.move_next();
                }
            }
            Token::ClosingBracket => return format_block(cursor, *inline),
            Token::Error => panic!("Got error: {cursor:?}"),
            _ => {}
        }
        cursor.move_next();
    }
}

/// Formats the found block.
///
/// After formatting, the cursor stands on the closing `}`, same as when called
fn format_block(cursor: &mut CursorMut<Token>, inline: bool) {
    // println!("At start of formatting: {:?}", cursor);
    // list looks like: "id //comment\n   {block}last"
    let mut is_empty = true;
    let mut one_line = inline;
    // Before anything, we want to split the list into block and last
    let last = cursor.split_after();
    // println!("last len: {}", last.len());
    // cursor is now at last elem in block. Advance by two
    cursor.move_next();
    debug_assert!(cursor.current().is_none());
    cursor.move_next();
    // First check facts about the block. (length, number of lines, empty etc)
    pre_process(cursor, &mut one_line, &mut is_empty);
    // println!("After first loop of formatting: {:?}", cursor);
    // remove trailing newlines and spaces before closing }
    // standing at ghost item, go back one
    debug_assert!(cursor.current().is_none());
    cursor.move_prev();
    // make sure we are standing on the closing bracket
    debug_assert!(matches!(cursor.current(), Some(Token::ClosingBracket)));

    while let Some(Token::NewLine | Token::Whitespace(_)) = cursor.peek_prev() {
        cursor.move_prev();
        cursor.remove_current();
    }
    debug_assert!(matches!(cursor.current(), Some(Token::ClosingBracket)));
    cursor.move_next();
    debug_assert!(cursor.current().is_none());
    cursor.move_next();
    debug_assert!(cursor.current().is_some());
    // Then format on second pass
    modify_block(cursor, one_line, is_empty);
    cursor.move_prev();
    // Assume standing on final closing bracket.
    debug_assert!(
        matches!(cursor.current(), Some(Token::ClosingBracket)),
        "token in formatting was {:?}",
        cursor,
    );
    // Add newline before if wanted
    if !one_line && !is_empty {
        cursor.insert_before(Token::NewLine);
    }
    debug_assert!(
        matches!(cursor.current(), Some(Token::ClosingBracket)),
        "token in formatting was {:?}",
        cursor,
    );

    // finally append "last" before returning
    // TODO: is this loop needed, or are we already at the end?
    while cursor.peek_next().is_some() {
        // println!("loop was needed");
        println!("skipped token: {:?}", cursor.current());
        cursor.move_next();
    }
    cursor.splice_after(last);
}

fn modify_block(cursor: &mut CursorMut<Token>, one_line: bool, is_empty: bool) {
    let mut in_block = false;
    while let Some(token) = cursor.current() {
        match token {
            Token::OpeningBracket => {
                // if first opening, add newline before and after if not collapsed
                if !in_block {
                    in_block = true;
                    if !one_line && !is_empty {
                        cursor.insert_before(Token::NewLine);
                        // TODO: What if there is a comment after the opening bracket?
                        cursor.insert_after(Token::NewLine);
                    } else {
                        while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                            cursor.move_prev();
                            cursor.remove_current();
                        }
                        cursor.insert_before(Token::Whitespace(" "));
                        if !is_empty {
                            cursor.insert_after(Token::Whitespace(" "));
                        }
                    }
                }
            }
            Token::ClosingBracket => {
                if one_line && !is_empty {
                    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                        cursor.move_prev();
                        cursor.remove_current();
                    }
                    cursor.insert_before(Token::Whitespace(" "));
                }
            } // Just include it?
            _ => {}
        }
        cursor.move_next();
    }
}

fn pre_process(cursor: &mut CursorMut<Token>, one_line: &mut bool, is_empty: &mut bool) {
    const MAX_LENGTH_ONELINE: usize = 72;

    let mut in_block = false;
    let mut on_second_line = false;
    let mut length: usize = 0;
    while let Some(token) = cursor.current() {
        match token {
            Token::Comment(_) => {
                *one_line = false;
                // If we see text inside the block, we know it's non-empty
                if in_block {
                    *is_empty = false;
                }
            }
            Token::NewLine => {
                if *is_empty {
                    cursor.remove_current();
                    cursor.move_prev();
                } else {
                    on_second_line = true;
                }
            }
            Token::OpeningBracket => {
                // LENDEF: How much space does a bracket add? space after so 2
                length += 2;
                in_block = true;
                // println!("When seeing opening bracket: {:?}", cursor);
            }
            Token::ClosingBracket => {
                // LENDEF: How much space does a closing bracket add? space before so 2
                length += 2;
            }
            Token::Whitespace(whitespace) => {
                // LENDEF
                length += whitespace.len();
                // Remove leading whitespace
                if *is_empty && in_block {
                    cursor.remove_current();
                    cursor.move_prev();
                }
            }
            Token::Text(text) => {
                // LENDEF:
                length += text.len();
                // If we see text inside the block, we know it's non-empty
                if in_block {
                    *is_empty = false;
                }
                // if we see text on the second line, we know it's not one line
                if on_second_line {
                    *one_line = false;
                }
            }
            Token::Equals => {
                // LENDEF: Space not included;
                length += 1;
            }
            Token::Error => todo!(),
        }
        cursor.move_next();
    }
    if length > MAX_LENGTH_ONELINE {
        *one_line = false;
    }
}
