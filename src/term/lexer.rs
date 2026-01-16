/*
   lib-laddertypes
   Copyright (C) 2023-2025  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use tiny_diagnostics::InputRegionTag;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LadderTypeToken {
    Symbol( String ),
    Char( char ),
    Num( i64 ),
    Open, OpenSpec, OpenSeq, OpenStruct,
    Close, CloseSpec, CloseSeq, CloseStruct,
    Univ, Ladder, EnumSep, StructSep,
    AssignType, SubType, TraitType, ParallelType,
    ArrowFunc, ArrowMorph
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
    Num{ sign: bool, val: i64 },
    Char( Option<char> ),
    Arrow( String ),
}

impl LexerState {
    fn into_token(self) -> Option<LadderTypeToken> {
        match self {
            LexerState::Any => None,
            LexerState::Sym(s) => Some(LadderTypeToken::Symbol(s)),
            LexerState::Num{sign, val} => Some(LadderTypeToken::Num( if sign{-val} else{val})),
            LexerState::Char(c) => Some(LadderTypeToken::Char(c?)),
            LexerState::Assign => Some(LadderTypeToken::AssignType),
            LexerState::Arrow(s) => match s.as_str() {
                "-->" => Some(LadderTypeToken::ArrowFunc),
                "-morph->" => Some(LadderTypeToken::ArrowMorph),
                _ => None
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub struct LadderTypeLexer<It>
where It: std::iter::Iterator<Item = char>
{
    chars: std::iter::Peekable<It>,
    pub position: usize,
    pub current_region: InputRegionTag
}

impl<It> LadderTypeLexer<It> where It: std::iter::Iterator<Item = char> {
    fn advance_region(&mut self) -> Option<char> {
        self.position += 1;
        self.current_region.end += 1;
        self.chars.next()
    }                                                                          
}

impl<It> From<It> for LadderTypeLexer<It>
where It: Iterator<Item = char>
{
    fn from(chars: It) -> Self {
        LadderTypeLexer {
            chars: chars.peekable(),
            position: 0,
            current_region: InputRegionTag::default()
        }
    }
}

impl<It> From<std::iter::Peekable<It>> for LadderTypeLexer<It>
where It: Iterator<Item = char>
{
    fn from(chars: std::iter::Peekable<It>) -> Self {
        LadderTypeLexer {
            chars,
            position: 0,
            current_region: InputRegionTag::default()
        }
    }
}

impl<It> Iterator for LadderTypeLexer<It>
where It: Iterator<Item = char>
{
    type Item = (InputRegionTag, Result<LadderTypeToken, LexError>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = LexerState::Any;

        self.current_region.begin = self.position;
        self.current_region.end   = self.position;

        while let Some(c) = self.chars.peek() {
            match &mut state {
                // determine token type
                LexerState::Any => {
                    match c {
                        
                        // terminate lexer on '=' since it must be a token of morphism-base not a ladder-type.
                        // todo: move this termination condition a layer up to a wrapped input iterator
                        '=' => { return None; },
                        '∀' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::Univ))); },
                        '(' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::Open))); },
                        ')' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::Close))); },
                        '<' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::OpenSpec))); },
                        '>' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::CloseSpec))); },
                        '[' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::OpenSeq))); },
                        ']' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::CloseSeq))); },
                        '{' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::OpenStruct))); },
                        '}' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::CloseStruct))); },
                        ';' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::StructSep))); },
                        '|' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::EnumSep))); },
                        '~' => { self.advance_region(); return Some((self.current_region, Ok(LadderTypeToken::Ladder))); },
                        '\'' => { self.advance_region(); state = LexerState::Char(None); },
                        ':' => {
                            self.advance_region();
                            state = LexerState::Assign;
                        },
                        '-' => { state = LexerState::Arrow(String::new()); },
                        c => {
                            if c.is_whitespace() {
                                self.advance_region();
                                self.current_region.begin += 1;
                            } else if c.is_alphabetic() {
                                state = LexerState::Sym( String::new() );
                            } else if c.is_digit(10) {
                                state = LexerState::Num{
                                    sign: false, val: 0
                                };
                            }
                        }
                    }
                }

                LexerState::Char(val) => {
                    *val = Some(
                        match self.advance_region() {
                            Some('\\') => {
                                match self.advance_region() {
                                    Some('0') => '\0',
                                    Some('n') => '\n',
                                    Some('t') => '\t',
                                    Some(c) => c,
                                    None => {
                                        return Some((self.current_region, Err(LexError::InvalidChar)));
                                    }
                                }
                            }
                            Some(c) => c,
                            None => {
                                return Some((self.current_region, Err(LexError::InvalidChar)));
                            }
                        });

                    match self.advance_region() {
                        Some('\'') => {
                            if let Some(token) = state.clone().into_token() {
                                return Some((self.current_region, Ok(token)));
                            }
                        }
                        _ => {
                            return Some((self.current_region, Err(LexError::InvalidChar)));
                        }
                    }
                }

                LexerState::Assign => {
                    match c {
                        '<' => {
                            // subtype
                            self.advance_region();
                            match self.advance_region() {
                                Some('=') => { return Some((self.current_region, Ok(LadderTypeToken::SubType))); },
                                Some(_) => { return Some((self.current_region, Err(LexError::InvalidChar))); },
                                None => { return Some((self.current_region, Err(LexError::InvalidChar))); }
                            }
                        }
                        '>' => {
                            // traittype
                            self.advance_region();
                            match self.advance_region() {
                                Some('<') => { return Some((self.current_region, Ok(LadderTypeToken::TraitType))); },
                                Some(_) => { return Some((self.current_region, Err(LexError::InvalidChar))); },
                                None => { return Some((self.current_region, Err(LexError::InvalidChar))); }
                            }
                        }
                        '|' => {
                            // paralleltype
                            self.advance_region();

                            match self.advance_region() {
                                Some('|') => { return Some((self.current_region, Ok(LadderTypeToken::ParallelType))); },
                                Some(_) => { return Some((self.current_region, Err(LexError::InvalidChar))); },
                                None => { return Some((self.current_region, Err(LexError::InvalidChar))); }
                            }
                        }
                        _ => {
                            return Some((self.current_region, Ok(LadderTypeToken::AssignType)));
                        }
                    }
                }

                LexerState::Arrow(s) => {
                    if c.is_digit(10) {
                        // encountered a signed number
                        state = LexerState::Num { sign: true, val: 0 };
                    } else {
                        let c = self.advance_region().unwrap();
                        s.push(c);
                        
                         if c == '>' {
                            // end of arrow
                            if let Some(token) = state.clone().into_token() {
                                return Some((self.current_region, Ok(token)));
                            }
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
                            return Some((self.current_region, Ok(token)));
                        }
                    } else {
                        // append to the current token

                        let c = self.advance_region().unwrap();

                        match &mut state {
                            LexerState::Sym(s) => {
                                s.push(c);
                            }
                            LexerState::Num{ sign, val } => {
                                if let Some(d) = c.to_digit(10) {
                                    *val = (*val) * 10 + d as i64;
                                } else {
                                    return Some((self.current_region, Err(LexError::InvalidDigit)));
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some(token) = state.into_token() {
            Some((self.current_region, Ok(token)))
        } else {
            None
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
