use super::{
    state_machines::{BlockSetting, Event, FormatterState},
    tokenizer, CursorMut, RemoveExt, Token,
};

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
            Token::ClosingBracket => {
                debug_assert!(
                    matches!(cursor.current(), Some(Token::ClosingBracket)),
                    "`format_block` started when not standing on a closing bracket"
                );
                let last = cursor.split_after();
                // cursor is now at last elem in block. Advance by two
                cursor.move_next();
                debug_assert!(cursor.current().is_none());
                cursor.move_next();
                format_block(cursor, *inline);
                cursor.splice_after(last);
                return;
            }
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
    /* Structure:
    When entering this function, the cursor should stand on the closing bracket of the block.
    The token stream has already been split to remove anything before and after the block.
    */
    let mut setting = BlockSetting::begin();
    pre_process(cursor, &mut setting);
    if !inline {
        setting.transition(Event::ShouldNotBeInline);
    }
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
    modify_block(cursor, &mut setting);
    // Assume standing on final closing bracket.
    debug_assert!(
        matches!(cursor.current(), Some(Token::ClosingBracket)),
        "token in formatting was {cursor:?}",
    );
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
}

fn modify_block(cursor: &mut CursorMut<Token>, settings: &mut BlockSetting) {
    let mut state = FormatterState::begin();
    while let Some(&mut token) = cursor.current() {
        match token {
            Token::OpeningBracket => match (&state, &settings) {
                (FormatterState::ReadingIdentifier, BlockSetting::OneLineEmpty) => {
                    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                        cursor.remove_prev();
                    }
                    cursor.insert_before(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                }
                (
                    FormatterState::ReadingIdentifier,
                    BlockSetting::MultiLineEmpty | BlockSetting::MultLine,
                ) => {
                    cursor.insert_before(Token::NewLine);
                    cursor.insert_after(Token::NewLine);
                }
                (FormatterState::ReadingIdentifier, BlockSetting::OneLine) => {
                    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                        cursor.remove_prev();
                    }
                    cursor.insert_before(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                    cursor.insert_after(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                }

                (
                    FormatterState::InBlock
                    | FormatterState::OnFirstLine
                    | FormatterState::OnSecondLine,
                    BlockSetting::MultLine,
                ) => (),

                (
                    FormatterState::InBlock
                    | FormatterState::OnFirstLine
                    | FormatterState::OnSecondLine,
                    BlockSetting::OneLineEmpty
                    | BlockSetting::MultiLineEmpty
                    | BlockSetting::OneLine,
                ) => todo!(),
            },
            Token::ClosingBracket => match settings {
                BlockSetting::OneLine => {
                    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
                        cursor.remove_prev();
                    }
                    cursor.insert_before(Token::Whitespace(tokenizer::Whitespace::Spaces(1)));
                }

                BlockSetting::OneLineEmpty
                | BlockSetting::MultiLineEmpty
                | BlockSetting::MultLine => (),
            },
            _ => {}
        }
        state.transition(token);
        cursor.move_next();
    }
    cursor.move_prev();
    // Add newline before closing `}` if wanted
    if matches!(settings, BlockSetting::MultLine) {
        cursor.insert_before(Token::NewLine);
    }
}

fn pre_process(cursor: &mut CursorMut<Token>, setting: &mut BlockSetting) {
    const MAX_LENGTH_ONELINE: usize = 72;
    let mut debug_string = String::new();

    let mut state = FormatterState::begin();
    let mut length: usize = 0;

    cursor.move_prev();
    cursor.move_prev();
    while let Some(Token::Whitespace(_)) = cursor.peek_prev() {
        cursor.remove_prev();
    }
    cursor.move_next();
    cursor.move_next();

    while let Some(&mut token) = cursor.current() {
        match &token {
            Token::Comment(_) => {
                if state.in_block() {
                    setting.transition(Event::CommentInBody);
                } else {
                    setting.transition(Event::CommentInID);
                }
            }
            Token::NewLine | Token::CRLF => {
                if setting.is_empty() {
                    cursor.remove_current();
                    cursor.move_prev();
                } else {
                    state.transition(token);
                }
            }
            Token::OpeningBracket => {
                // LENDEF: How much space does a bracket add? space before and after so 3
                length += 3;
                debug_string.push_str("added {: 3\n");
                if state.in_block() {
                    setting.transition(Event::ShouldNotBeInline);
                }
                state.transition(token);
                // println!("When seeing opening bracket: {:?}", cursor);
            }
            Token::ClosingBracket => {
                // LENDEF: How much space does a closing bracket add? space before so 2
                length += 2;
                debug_string.push_str("added }: 2\n");
            }
            Token::Whitespace(whitespace) => {
                // Remove leading whitespace
                if setting.is_empty() && state.in_block() {
                    cursor.remove_current();
                    cursor.move_prev();
                } else if state.in_block() {
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
                if state.in_block() {
                    setting.transition(Event::TextInBody);
                }
                // if we see text on the second line, we know it's not one line
                if state.on_second_line() {
                    setting.transition(Event::ShouldNotBeInline);
                }
            }
            Token::Equals => {
                // LENDEF: Space not included;
                debug_string.push_str("added =: 1\n");
                length += 1;
            }
            Token::Error => todo!(),
        }
        state.transition(token);
        cursor.move_next();
    }
    if length > MAX_LENGTH_ONELINE {
        // println!("{debug_string}");
        // println!("length was {length}");
        setting.transition(Event::ShouldNotBeInline);
    }
}
