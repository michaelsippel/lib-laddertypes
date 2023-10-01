
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LadderTypeToken {
    Symbol( String ),
    Char( char ),
    Num( i64 ),
    Open,
    Close,
    Ladder,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LexError {
    /// found a non-digit character inside a numeric token
    InvalidDigit,

    /// quoted character token didnt close correctly with '
    InvalidChar,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(PartialEq, Eq, Clone, Debug)]
enum LexerState {
    Any,
    Sym( String ),
    Num( i64 ),
    Char( Option<char> )
}

impl LexerState {
    fn into_token(self) -> Option<LadderTypeToken> {
        match self {
            LexerState::Any => None,
            LexerState::Sym(s) => Some(LadderTypeToken::Symbol(s)),
            LexerState::Num(n) => Some(LadderTypeToken::Num(n)),
            LexerState::Char(c) => Some(LadderTypeToken::Char(c?))
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct LadderTypeLexer<It>
where It: std::iter::Iterator<Item = char>
{
    chars: std::iter::Peekable<It>,
}

impl<It> From<It> for LadderTypeLexer<It>
where It: Iterator<Item = char>
{
    fn from(chars: It) -> Self {
        LadderTypeLexer {
            chars: chars.peekable()
        }
    }
}

impl<It> Iterator for LadderTypeLexer<It>
where It: Iterator<Item = char>
{
    type Item = Result<LadderTypeToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = LexerState::Any;

        while let Some(c) = self.chars.peek() {
            match &mut state {

                // determine token type
                LexerState::Any => {
                    match c {
                        '<' => { self.chars.next(); return Some(Ok(LadderTypeToken::Open)); },
                        '>' => { self.chars.next(); return Some(Ok(LadderTypeToken::Close)); },
                        '~' => { self.chars.next(); return Some(Ok(LadderTypeToken::Ladder)); },
                        '\'' => { self.chars.next(); state = LexerState::Char(None); },
                        ' ' => { self.chars.next(); },
                        c => {
                            if c.is_alphabetic() {
                                state = LexerState::Sym( String::new() );
                            } else if c.is_digit(10) {
                                state = LexerState::Num( 0 );
                            }
                        }
                    }
                }

                LexerState::Char(val) => {
                    // todo escape characters
                    *val = self.chars.next();

                    match self.chars.next() {
                        Some('\'') => {
                            if let Some(token) = state.clone().into_token() {
                                return Some(Ok(token));
                            }                            
                        }
                        _ => {
                            return Some(Err(LexError::InvalidChar));
                        }
                    }
                }

                _ => {

                    if c.is_whitespace() || *c == '>' || *c == '~' {
                        // finish the current token

                        if let Some(token) = state.clone().into_token() {
                            return Some(Ok(token));
                        }
                    } else {
                        // append to the current token

                        let c = self.chars.next().unwrap();

                        match &mut state {
                            LexerState::Sym(s) => {
                                s.push(c);
                            }

                            LexerState::Num(n) => {
                                if let Some(d) = c.to_digit(10) {
                                    *n = (*n) * 10 + d as i64;
                                } else {
                                    return Some(Err(LexError::InvalidDigit));
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some(token) = state.into_token() {
            Some(Ok(token))
        } else {
            None
        }
    }
}

