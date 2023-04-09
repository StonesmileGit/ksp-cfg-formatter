/// block formatting code
pub mod format_blocks;
mod state_machines;
use super::{Indentation, LineReturn};
use format_blocks::format_blocks;

/// Tokenizer documentation
pub mod tokenizer;
use tokenizer::Token;

use logos::Logos;
use std::collections::{linked_list::CursorMut, LinkedList};
use std::time::Instant;

trait RemoveExt<T> {
    fn remove_prev(&mut self) -> Option<T>;
    fn remove_next(&mut self) -> Option<T>;
}

impl<'a, T> RemoveExt<T> for CursorMut<'a, T> {
    /// Removes the item sitting before the current item. After removing, the cursor points to the same item as before calling this.
    fn remove_prev(&mut self) -> Option<T> {
        self.move_prev();
        self.remove_current()
    }
    /// Removes the item sitting after the current item. After removing, the cursor points to the same item as before calling this.
    fn remove_next(&mut self) -> Option<T> {
        self.move_next();
        let val = self.remove_current();
        self.move_prev();
        val
    }
}

/// Struct for holding the settings to use for formatting. use `self.format_text()` to format text
///
/// Example:
/// ```
/// use ksp_cfg_formatter::{token_formatter::Formatter, Indentation, LineReturn};
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
    /// use ksp_cfg_formatter::{token_formatter::Formatter, Indentation, LineReturn};
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
    /// use ksp_cfg_formatter::{token_formatter::Formatter, Indentation, LineReturn};
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
        let debug_print = false;
        let total = Instant::now();

        let start = Instant::now();
        let mut token_list = Token::lexer(text).collect::<LinkedList<Token>>();
        let tokenize_time = start.elapsed();

        // dbg!(&token_list);

        let balanced_brackets = check_brackets(&token_list);
        if !balanced_brackets {
            return text.to_owned();
        }

        let formatting_timer = Instant::now();

        let start = Instant::now();
        let original_line_ending = check_for_crlf(&mut token_list.cursor_front_mut());
        let check_crlf_time = start.elapsed();

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

        let start = Instant::now();
        if self.line_return != LineReturn::LF && original_line_ending == Token::CRLF {
            set_line_endings_to_crlf(&mut token_list.cursor_front_mut());
        }
        let change_crlf_time = start.elapsed();

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
            println!("{check_crlf_time:?} Checking for CRLF");
            println!("{leading_whitespace_time:?} Removed leading whitespace");
            println!("{format_blocks_time:?} Formatted blocks");
            println!("{indentation_time:?} Fixed indentation");
            println!("{trailing_whitespace_time:?} Removed trailing whitespace");
            println!("{newlines_time:?} Trailing whitelines");
            println!("{change_crlf_time:?} Converting newline chars");
            println!("{to_string_time:?} Converting to string");
            println!("{formatting_time:?} Total formatting");
            println!("{total_time:?} Total");
        }
        text
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

fn check_for_crlf<'a>(cursor: &mut CursorMut<Token>) -> Token<'a> {
    let mut found_crlf = false;
    while let Some(token) = cursor.current() {
        if token == &mut Token::CRLF {
            found_crlf = true;
            cursor.insert_after(Token::NewLine);
            cursor.remove_current();
        }
        cursor.move_next();
    }
    if found_crlf {
        Token::CRLF
    } else {
        Token::NewLine
    }
}

fn set_line_endings_to_crlf(cursor: &mut CursorMut<Token>) {
    while let Some(token) = cursor.current() {
        if token == &mut Token::NewLine {
            cursor.insert_after(Token::CRLF);
            cursor.remove_current();
        }
        cursor.move_next();
    }
}

/// Removes leading whitespace
pub fn remove_leading_whitespace(cursor: &mut CursorMut<Token>) {
    while let Some(Token::Whitespace(_)) = cursor.current() {
        cursor.remove_current();
    }
    while let Some(token) = cursor.current() {
        if matches!(token, Token::NewLine) {
            while matches!(cursor.peek_next(), Some(Token::Whitespace(_))) {
                cursor.remove_next();
            }
        }
        cursor.move_next();
    }
}

/// Removes trailing whitespace
pub fn remove_trailing_whitespace(cursor: &mut CursorMut<Token>) {
    while let Some(token) = cursor.current() {
        if matches!(token, Token::NewLine) {
            while matches!(cursor.peek_prev(), Some(Token::Whitespace(_))) {
                cursor.remove_prev();
            }
        }
        cursor.move_next();
    }
    while matches!(cursor.peek_prev(), Some(Token::Whitespace(_))) {
        cursor.remove_prev();
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
                if level > 0 {
                    let indent_text = match indentation {
                        Indentation::Spaces(n) => {
                            Token::Whitespace(tokenizer::Whitespace::Spaces(n * level))
                        }
                        Indentation::Tabs => Token::Whitespace(tokenizer::Whitespace::Tabs(level)),
                    };
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
