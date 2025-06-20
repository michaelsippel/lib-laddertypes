
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LadderTypeToken {
    Symbol( String ),
    Char( char ),
    Num( i64 ),
    Univ,
    Open, OpenSpec, OpenSeq, OpenStruct,
    Close, CloseSpec, CloseSeq, CloseStruct,
    Ladder,
    EnumSep, StructSep,
    AssignType,
    AssignSubType,
    AssignTraitType,
    AssignParallelType,
    // todo: Func, Morph
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LexError {
    /// found a non-digit character inside a numeric token
    InvalidDigit,

    /// quoted character token didnt close correctly with '
    InvalidChar,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(PartialEq, Eq, Clone, Debug)]
enum LexerState {
    Any,
    Assign,
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
            LexerState::Char(c) => Some(LadderTypeToken::Char(c?)),
            LexerState::Assign => Some(LadderTypeToken::AssignType),
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

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
                        '∀' => { self.chars.next(); return Some(Ok(LadderTypeToken::Univ)); },
                        '(' => { self.chars.next(); return Some(Ok(LadderTypeToken::Open)); },
                        ')' => { self.chars.next(); return Some(Ok(LadderTypeToken::Close)); },
                        '<' => { self.chars.next(); return Some(Ok(LadderTypeToken::OpenSpec)); },
                        '>' => { self.chars.next(); return Some(Ok(LadderTypeToken::CloseSpec)); },
                        '[' => { self.chars.next(); return Some(Ok(LadderTypeToken::OpenSeq)); },
                        ']' => { self.chars.next(); return Some(Ok(LadderTypeToken::CloseSeq)); },
                        '{' => { self.chars.next(); return Some(Ok(LadderTypeToken::OpenStruct)); },
                        '}' => { self.chars.next(); return Some(Ok(LadderTypeToken::CloseStruct)); },
                        ';' => { self.chars.next(); return Some(Ok(LadderTypeToken::StructSep)); },
                        '|' => { self.chars.next(); return Some(Ok(LadderTypeToken::EnumSep)); },
                        '~' => { self.chars.next(); return Some(Ok(LadderTypeToken::Ladder)); },
                        '\'' => { self.chars.next(); state = LexerState::Char(None); },
                        ':' => {
                            self.chars.next();
                            state = LexerState::Assign;
                        },
                        c => {
                            if c.is_whitespace() {
                                self.chars.next();
                            } else if c.is_alphabetic() {
                                state = LexerState::Sym( String::new() );
                            } else if c.is_digit(10) {
                                state = LexerState::Num( 0 );
                            }
                        }
                    }
                }

                LexerState::Char(val) => {
                    *val = Some(
                        match self.chars.next() {
                            Some('\\') => {
                                match self.chars.next() {
                                    Some('0') => '\0',
                                    Some('n') => '\n',
                                    Some('t') => '\t',
                                    Some(c) => c,
                                    None => {
                                        return Some(Err(LexError::InvalidChar));
                                    }
                                }
                            }
                            Some(c) => c,
                            None => {
                                return Some(Err(LexError::InvalidChar));
                            }
                        });

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

                LexerState::Assign => {
                    match c {
                        '<' => {
                            // subtype
                            self.chars.next();
                            match self.chars.next() {
                                Some('=') => { return Some(Ok(LadderTypeToken::AssignSubType)); },
                                Some(_) => { return Some(Err(LexError::InvalidChar)); },
                                None => { return Some(Err(LexError::InvalidChar)); }
                            }
                        }
                        '>' => {
                            // traittype
                            self.chars.next();
                            match self.chars.next() {
                                Some('<') => { return Some(Ok(LadderTypeToken::AssignTraitType)); },
                                Some(_) => { return Some(Err(LexError::InvalidChar)); },
                                None => { return Some(Err(LexError::InvalidChar)); }
                            }
                        }
                        '|' => {
                            // paralleltype
                            self.chars.next();

                            match self.chars.next() {
                                Some('|') => { return Some(Ok(LadderTypeToken::AssignParallelType)); },
                                Some(_) => { return Some(Err(LexError::InvalidChar)); },
                                None => { return Some(Err(LexError::InvalidChar)); }
                            }
                        }
                        _ => {
                            return Some(Ok(LadderTypeToken::AssignType));
                        }
                    }
                }

                _ => {

                    if c.is_whitespace()
                    || *c == ')' || *c == '>' || *c == ']' || *c=='}'
                    || *c == '~' || *c==':' || *c==';' || *c=='|'
                    {
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

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
