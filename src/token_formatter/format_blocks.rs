use super::{tokenizer, CursorMut, RemoveExt, Token};

/// blah
/// # Panics
/// This function panics if any token in the stream is an Error
pub fn format_blocks(cursor: &mut CursorMut<Token>, top_level: &bool, inline: &bool) {
    // println!("cursor pos: {}", cursor.index().unwrap());
    let mut in_block = *top_level;
    while let Some(token) = cursor.current() {
        match token {
            Token::OpeningBracket => {
                if !in_block {
                    in_block = true;
                    cursor.move_next();
                    continue;
                }
                // println!("found block");
                // We have reached the beginning of a new block. Go back and find the identifier, split the list and start recursion
                while let Some(token) = cursor.current() {
                    match token {
                        Token::Text(_) => break,
                        _ => cursor.move_prev(),
                    }
                }
                while let Some(token) = cursor.current() {
                    match token {
                        Token::NewLine => break,
                        _ => cursor.move_prev(),
                    }
                }
                cursor.move_next();
                while let Some(token) = cursor.current() {
                    match token {
                        Token::Whitespace(_) => cursor.move_next(),
                        _ => break,
                    }
                }
                let first = cursor.split_before();
                format_blocks(cursor, &false, inline);
                debug_assert!(
                    matches!(cursor.current(), Some(Token::ClosingBracket)),
                    "token was {cursor:?}",
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
        cursor.remove_prev();
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
        "token in formatting was {cursor:?}",
    );
    // Add newline before if wanted
    if !one_line && !is_empty {
        cursor.insert_before(Token::NewLine);
    }
    debug_assert!(
        matches!(cursor.current(), Some(Token::ClosingBracket)),
        "token in formatting was {cursor:?}",
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
                    if one_line {
                        while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                            cursor.remove_prev();
                        }
                        cursor.insert_before(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                        if !is_empty {
                            cursor
                                .insert_after(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                        }
                    } else {
                        cursor.insert_before(Token::NewLine);
                        // TODO: What if there is a comment after the opening bracket?
                        cursor.insert_after(Token::NewLine);
                    }
                }
            }
            Token::ClosingBracket => {
                if one_line && !is_empty {
                    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                        cursor.remove_prev();
                    }
                    cursor.insert_before(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                }
            }
            _ => {}
        }
        cursor.move_next();
    }
}

fn pre_process(cursor: &mut CursorMut<Token>, one_line: &mut bool, is_empty: &mut bool) {
    const MAX_LENGTH_ONELINE: usize = 72;
    let mut debug_string = String::new();

    cursor.move_prev();
    cursor.move_prev();
    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
        cursor.remove_prev();
    }
    cursor.move_next();
    cursor.move_next();

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
            Token::NewLine | Token::CRLF => {
                if *is_empty {
                    cursor.remove_current();
                    cursor.move_prev();
                } else {
                    on_second_line = true;
                }
            }
            Token::OpeningBracket => {
                // LENDEF: How much space does a bracket add? space before and after so 3
                length += 3;
                debug_string.push_str("added {: 3\n");
                if in_block {
                    *one_line = false;
                }
                in_block = true;
                // println!("When seeing opening bracket: {:?}", cursor);
            }
            Token::ClosingBracket => {
                // LENDEF: How much space does a closing bracket add? space before so 2
                length += 2;
                debug_string.push_str("added }: 2\n");
            }
            Token::Whitespace(whitespace) => {
                // Remove leading whitespace
                if *is_empty && in_block {
                    cursor.remove_current();
                    cursor.move_prev();
                } else if in_block {
                    // LENDEF
                    let whitelen = match whitespace {
                        tokenizer::Whitespace::Spaces(n) => *n,
                        tokenizer::Whitespace::Tabs(n) => 4 * *n,
                    };
                    length += whitelen;
                    debug_string.push_str(format!("added whitespace {whitelen}\n").as_str());
                }
            }
            Token::Text(text) => {
                // LENDEF:
                let textlen = text.chars().count();
                length += &textlen;
                debug_string.push_str(format!("added text of len {textlen}\n").as_str());
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
                debug_string.push_str("added =: 1\n");
                length += 1;
            }
            Token::Error => todo!(),
        }
        cursor.move_next();
    }
    if length > MAX_LENGTH_ONELINE {
        // println!("{debug_string}");
        // println!("length was {length}");
        *one_line = false;
    }
}
