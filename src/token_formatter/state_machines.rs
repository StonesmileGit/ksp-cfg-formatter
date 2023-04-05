use super::Token;

pub enum BlockSetting {
    OneLineEmpty,
    MultiLineEmpty,
    OneLine,
    MultLine,
}

impl BlockSetting {
    pub const fn begin() -> Self {
        Self::OneLineEmpty
    }
    pub fn transition(&mut self, event: Event) {
        match (&self, event) {
            (Self::OneLineEmpty, Event::CommentInID) => *self = Self::MultiLineEmpty,
            (Self::OneLineEmpty, Event::CommentInBody) => *self = Self::MultLine,
            (Self::OneLineEmpty, Event::TextInBody) => *self = Self::OneLine,
            (Self::MultiLineEmpty, Event::TextInBody) => *self = Self::MultLine,
            (Self::MultiLineEmpty, Event::CommentInBody) => *self = Self::MultLine,
            (Self::OneLine, Event::CommentInID | Event::CommentInBody) => *self = Self::MultLine,
            (Self::OneLine, Event::ShouldNotBeInline) => *self = Self::MultLine,

            (Self::OneLineEmpty, Event::ShouldNotBeInline) => (),
            (Self::MultiLineEmpty, Event::ShouldNotBeInline) => (),
            (Self::OneLine, Event::TextInBody) => (),
            (Self::MultLine, Event::CommentInBody) => (),
            (Self::MultLine, Event::TextInBody) => (),
            (Self::MultLine, Event::ShouldNotBeInline) => (),

            (Self::MultiLineEmpty, Event::CommentInID) => panic!("This shouldn't happen"),
            (Self::MultLine, Event::CommentInID) => panic!("This shouldn't happen"),
        }
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::MultiLineEmpty | Self::OneLineEmpty)
    }
}
pub enum Event {
    CommentInID,
    CommentInBody,
    TextInBody,
    ShouldNotBeInline,
}

pub enum FormatterState {
    ReadingIdentifier,
    InBlock,
    OnFirstLine,
    OnSecondLine,
}

impl FormatterState {
    pub const fn begin() -> Self {
        Self::ReadingIdentifier
    }
    pub fn transition(&mut self, token: Token) {
        match (&self, token) {
            (_, Token::Error) => todo!(),
            (Self::ReadingIdentifier, Token::OpeningBracket) => *self = Self::InBlock,
            (Self::ReadingIdentifier, Token::ClosingBracket) => {
                panic!("WOAH, that shouldn't happen")
            }
            (Self::InBlock, Token::NewLine) => *self = Self::OnFirstLine,
            (Self::OnFirstLine, Token::NewLine) => *self = Self::OnSecondLine,

            (Self::ReadingIdentifier, Token::Comment(_)) => (),
            (Self::ReadingIdentifier, Token::NewLine) => (),
            (Self::ReadingIdentifier, Token::Whitespace(_)) => (),
            (Self::ReadingIdentifier, Token::Text(_)) => (),
            (Self::InBlock, Token::Comment(_)) => (),
            (Self::InBlock, Token::OpeningBracket) => (),
            (Self::InBlock, Token::ClosingBracket) => (),
            (Self::InBlock, Token::Whitespace(_)) => (),
            (Self::InBlock, Token::Equals) => (),
            (Self::InBlock, Token::Text(_)) => (),
            (Self::OnSecondLine, Token::Comment(_)) => (),
            (Self::OnSecondLine, Token::NewLine) => (),
            (Self::OnSecondLine, Token::CRLF) => (),
            (Self::OnSecondLine, Token::OpeningBracket) => (),
            (Self::OnSecondLine, Token::ClosingBracket) => (),
            (Self::OnSecondLine, Token::Whitespace(_)) => (),
            (Self::OnSecondLine, Token::Equals) => (),
            (Self::OnSecondLine, Token::Text(_)) => (),

            (Self::InBlock, Token::CRLF) => todo!(),
            (Self::ReadingIdentifier, Token::Equals) => todo!(),
            (Self::ReadingIdentifier, Token::CRLF) => todo!(),

            (Self::OnFirstLine, Token::Comment(_)) => (),
            (Self::OnFirstLine, Token::CRLF) => todo!(),
            (Self::OnFirstLine, Token::OpeningBracket) => (),
            (Self::OnFirstLine, Token::ClosingBracket) => (),
            (Self::OnFirstLine, Token::Whitespace(_)) => (),
            (Self::OnFirstLine, Token::Equals) => (),
            (Self::OnFirstLine, Token::Text(_)) => (),
        }
    }

    pub const fn in_block(&self) -> bool {
        matches!(self, Self::InBlock | Self::OnSecondLine | Self::OnFirstLine)
    }

    pub const fn on_second_line(&self) -> bool {
        matches!(self, Self::OnSecondLine)
    }
}
