use lazy_static::lazy_static;
use regex::Regex;
use std::time::{Duration, Instant};
/// Defines which End of Line sequence to be used
///
/// Can have the values `LF`, `CRLF` or `Identify`.
///
/// When using `Identify`, the formatter tries to figure out what sequence to use, based on the provided text.
///
/// Example:
/// ```
/// use ksp_cfg_formatter::char_formatter::{Formatter, Indentation, LineReturn};
///
/// let line_return = LineReturn::LF;
///
/// let indentation = Indentation::Tabs;
/// let formatter = Formatter::new(indentation, false, line_return);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
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
/// use ksp_cfg_formatter::char_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Spaces(4);
///
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false, line_return);
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

impl From<Option<usize>> for Indentation {
    fn from(setting: Option<usize>) -> Self {
        setting.map_or(Self::Tabs, Self::Spaces)
    }
}

/// Struct for holding the settings to use for formatting. use `self.format_text()` to format text
///
/// Example:
/// ```
/// use ksp_cfg_formatter::char_formatter::{Formatter, Indentation, LineReturn};
///
/// let indentation = Indentation::Tabs;
/// let line_return = LineReturn::Identify;
/// let formatter = Formatter::new(indentation, false, line_return);
/// # // this is needed to test the code, but not important to readers
/// # let input = "";
/// let output = formatter.format_text(input);
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
    /// use ksp_cfg_formatter::char_formatter::{Formatter, Indentation, LineReturn};
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

    // Formats the provided text according to the settings of the Formatter
    /// Takes the provided text and formats it according to the settings of the `Formatter`
    ///
    /// TODO: Explain the parts of the formatter.
    ///
    /// Example:
    /// ```
    /// use ksp_cfg_formatter::char_formatter::{Formatter, Indentation, LineReturn};
    ///
    /// let indentation = Indentation::Tabs;
    /// let line_return = LineReturn::Identify;
    /// let formatter = Formatter::new(indentation, false, line_return);
    /// # // this is needed to test the code, but not important to readers
    /// # let input = "";
    /// let output = formatter.format_text(input);
    /// ```
    #[must_use]
    pub fn format_text(&self, text: &str) -> String {
        let mut indent = 0;
        for char in text.chars() {
            if char == '{' {
                indent += 1;
            }
            if char == '}' {
                indent -= 1;
            }
            if indent < 0 {
                return text.to_owned();
            }
        }
        if indent != 0 {
            return text.to_owned();
        }
        let debug_print = false;
        let total = Instant::now();
        let start = Instant::now();
        let mut output = Self::remove_leading_whitespace(text);
        let leading_whitespace_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let start = Instant::now();
        output = Self::format_blocks(None, &self.inline, &mut output.chars());
        let format_blocks_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let start = Instant::now();
        output = Self::indentation(&output, self.indentation);
        let indentation_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let start = Instant::now();
        output = Self::remove_trailing_whitespace(&output);
        let trailing_whitespace_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let start = Instant::now();
        output = Self::remove_leading_and_trailing_newlines(&output);
        let newlines_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let start = Instant::now();
        output = Self::crlf(&output, self.line_return);
        let crlf_time = if debug_print {
            start.elapsed()
        } else {
            Duration::default()
        };
        let total_time = if debug_print {
            total.elapsed()
        } else {
            Duration::default()
        };
        if debug_print {
            println!("{leading_whitespace_time:?} Removed leading whitespace");
            println!("{format_blocks_time:?} Formatted blocks");
            println!("{indentation_time:?} Fixed indentation");
            println!("{trailing_whitespace_time:?} Removed trailing whitespace");
            println!("{newlines_time:?} Trailing whitelines");
            println!("{crlf_time:?} CRLF");
            println!("{total_time:?} Total");
        }
        output
    }

    /// Sets the `EoL` sequence based on settings
    #[must_use]
    pub fn crlf(text: &str, line_return: LineReturn) -> String {
        let mut line_return_local = &line_return;
        if line_return == LineReturn::Identify {
            line_return_local = if text.contains('\r') {
                &LineReturn::CRLF
            } else {
                &LineReturn::LF
            }
        }
        let intermediate = text.replace("\r\n", "\n");
        if line_return_local == &LineReturn::LF {
            return intermediate;
        }
        intermediate.replace('\n', "\r\n")
    }

    /// takes an un-indented text and indents it based on open blocks
    #[must_use]
    pub fn indentation(text: &str, indentation: Indentation) -> String {
        let indentation_string = indentation.to_string();
        let mut output: String = String::new();
        let mut indentation: usize = 0;
        let mut last_char = char::default();
        let mut char_iter = text.chars();
        while let Some(char) = char_iter.next() {
            let mut curr_char = char;
            match (last_char, char) {
                // This is a comment. Aggregate all of the comment
                ('/', '/') => {
                    output.push(char);
                    output.push_str(&Self::aggregate_comment(&mut char_iter));
                    curr_char = '\n';
                }
                // We are opening a block, increase indentation
                ('\n', '{') => {
                    output.push_str(&indentation_string.repeat(indentation));
                    indentation += 1;
                    output.push(char);
                }
                (_, '{') => {
                    indentation += 1;
                    output.push(char);
                }
                // We are closing a block, reduce indentation
                ('\n', '}') => {
                    debug_assert!(indentation > 0);
                    indentation -= 1;
                    output.push_str(&indentation_string.repeat(indentation));
                    output.push(char);
                }
                (_, '}') => {
                    debug_assert!(indentation > 0);
                    indentation -= 1;
                    output.push(char);
                }
                // newline, add indentation before continuing
                ('\n', _) => {
                    output.push_str(&indentation_string.repeat(indentation));
                    output.push(char);
                }
                // Just add any other char to the output
                _ => output.push(char),
            }
            last_char = curr_char;
        }
        output
    }

    /// Removes tabs and spaces at the end of a line
    #[must_use]
    pub fn remove_trailing_whitespace(text: &str) -> String {
        lazy_static! {
            static ref PATTERN: Regex = Regex::new(r"(?m)[\t ]+\r*$").unwrap();
        }
        PATTERN.replace_all(text, "").to_string()
    }

    /// Removes tabs and spaces at the beginning of every line
    #[must_use]
    pub fn remove_leading_whitespace(text: &str) -> String {
        lazy_static! {
            static ref PATTERN: Regex = Regex::new(r"(?m)^[\t ]+").unwrap();
        }
        PATTERN.replace_all(text, "").to_string()
    }

    /// Removes leading and trailing newlines
    #[must_use]
    pub fn remove_leading_and_trailing_newlines(text: &str) -> String {
        lazy_static! {
            static ref START_PATTERN: Regex = Regex::new(r"^(\r*\n)*").unwrap();
        }
        let intermediate = START_PATTERN.replace_all(text, "");
        lazy_static! {
            static ref END_PATTERN: Regex = Regex::new(r"(\r*\n)*$").unwrap();
        }
        END_PATTERN.replace_all(&intermediate, "\n").to_string()
    }

    /// Formats the blocks of the text based on the settings
    pub fn format_blocks(
        maybe_identifier: Option<&String>,
        inline: &bool,
        char_iter: &mut std::str::Chars,
    ) -> String {
        let mut output = String::new();
        let mut potential_identifier = String::new();
        let mut last_char = char::default();
        while let Some(char) = char_iter.next() {
            match (last_char, char) {
                // This is a comment. aggregate the whole comment and push that all at once
                ('/', '/') => {
                    let comment = Self::aggregate_comment(char_iter);
                    potential_identifier.push(char);
                    potential_identifier.push_str(&comment);
                }
                // Starting a block
                (_, '{') => {
                    // Send identifier to block handler and recursively call format_blocks
                    let (to_out, ident) = Self::get_valid_identifier(&potential_identifier);
                    output.push_str(to_out);
                    output.push_str(&Self::format_blocks(
                        Some(&ident.to_owned()),
                        inline,
                        char_iter,
                    ));
                    potential_identifier.clear();
                }
                // Closing a block
                (_, '}') => {
                    break;
                }
                _ => potential_identifier.push(char),
            }
            last_char = char;
        }

        // Flush any partial identifier that was started
        output.push_str(&potential_identifier);

        match maybe_identifier {
            Some(identifier) => Self::format_block(identifier, &output, *inline),
            // This is the top level outside any blocks
            None => output,
        }
    }

    fn aggregate_comment(char_iter: &mut std::str::Chars) -> String {
        let mut comment = String::new();
        for char in char_iter {
            if char == '\n' {
                comment.push(char);
                break;
            }
            comment.push(char);
        }
        comment
    }

    fn get_valid_identifier(potential_identifier: &str) -> (&str, &str) {
        lazy_static! {
            static ref VALID_ID: Regex =
                Regex::new(r"[a-zA-Z0-9@!%&][a-zA-Z0-9/ \t\n\r\[\],\*\+\-\?]*(//.*)?[\n \t]*$")
                    .unwrap();
        }
        let splits: Vec<&str> = VALID_ID.split(potential_identifier).collect();
        let first = splits.first().unwrap();
        let second = VALID_ID
            .find(potential_identifier)
            .map_or_else(|| panic!("Got error in identifier"), |val| val.as_str());
        (first, second)
    }

    fn format_block(identifier: &str, body: &str, inline: bool) -> String {
        lazy_static! {
            static ref WHITESPACE_PATTERN: Regex =
                Regex::new(r"^[ \t\r\n]*|[ \t\r\n]*$").expect("This Regex should compile");
        }
        let id: String = WHITESPACE_PATTERN.replace_all(identifier, "").to_string();
        let body: String = WHITESPACE_PATTERN.replace_all(body, "").to_string();
        match Self::should_be_collapsed(&id, &body, &mut inline.clone()) {
            ShouldCollapse::Yes => format!("{id} {{ {body} }}"),
            ShouldCollapse::No => format!("{id}\n{{\n{body}\n}}"),
            ShouldCollapse::Empty => format!("{id} {{}}"),
        }
    }

    fn should_be_collapsed(
        identifier: &str,
        body: &str,
        should_collapse: &mut bool,
    ) -> ShouldCollapse {
        if body.is_empty() {
            return ShouldCollapse::Empty;
        }
        *should_collapse &= !identifier.contains("//");
        *should_collapse &= !body.contains("//");
        *should_collapse &= !body.contains('\n');
        let max_length: usize = 72;
        let length: usize = identifier.len() + body.len();
        *should_collapse &= length < max_length;
        if *should_collapse {
            ShouldCollapse::Yes
        } else {
            ShouldCollapse::No
        }
    }
}

enum ShouldCollapse {
    Empty,
    Yes,
    No,
}
