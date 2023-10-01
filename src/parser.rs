use {
    std::iter::Peekable,
    crate::{
        dict::*,
        term::*,
        lexer::*
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedClose,
    UnexpectedLadder,
    UnexpectedEnd,
    UnexpectedToken
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl std::str::FromStr for TypeTerm {
    type Err = ParseError;

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        // creating a new context every time is not that useful..
        let mut dict = TypeDict::new();
        dict.parse(&mut LadderTypeLexer::from(s.chars()).peekable())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeDict {
    fn parse_app<It>( &mut self, tokens: &mut Peekable<LadderTypeLexer<It>> ) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut args = Vec::new();
        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Close) => {
                    tokens.next();
                    return Ok(TypeTerm::App(args));
                }
                _ => {
                    match self.parse_partial(tokens) {
                        Ok(a) => { args.push(a); }
                        Err(err) => { return Err(err); }
                    }
                }
            }
        }
        Err(ParseError::UnexpectedEnd)
    }

    fn parse_rung<It>( &mut self, tokens: &mut Peekable<LadderTypeLexer<It>> ) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        match tokens.next() {
            Some(Ok(LadderTypeToken::Open)) => self.parse_app(tokens),
            Some(Ok(LadderTypeToken::Close)) => Err(ParseError::UnexpectedClose),
            Some(Ok(LadderTypeToken::Ladder)) => Err(ParseError::UnexpectedLadder),
            Some(Ok(LadderTypeToken::Symbol(s))) =>
                Ok(TypeTerm::TypeID(
                    if let Some(tyid) = self.get_typeid(&s) {
                        tyid
                    } else {
                        self.add_typename(s)
                    }
                )),
            Some(Ok(LadderTypeToken::Char(c))) => Ok(TypeTerm::Char(c)),
            Some(Ok(LadderTypeToken::Num(n))) => Ok(TypeTerm::Num(n)),
            Some(Err(err)) => Err(ParseError::LexError(err)),
            None => Err(ParseError::UnexpectedEnd)
        }
    }
    
    fn parse_partial<It>( &mut self, tokens: &mut Peekable<LadderTypeLexer<It>> ) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        let mut rungs = Vec::new();

        match self.parse_rung(tokens) {
            Ok(t) => { rungs.push(t); }
            Err(err) => { return Err(err); }
        }
        
        while let Some(tok) = tokens.peek() {
            match tok {
                Ok(LadderTypeToken::Ladder) => {
                    tokens.next();

                    if rungs.len() > 0 {
                        match self.parse_rung(tokens) {
                            Ok(t) => { rungs.push(t); }
                            Err(err) => { return Err(err); }
                        }
                    } else {
                        return Err(ParseError::UnexpectedLadder);    
                    }
                }
                Err(lexerr) => {
                    return Err(ParseError::LexError(lexerr.clone()));
                }
                _ => {
                    break;
                }
            }
        }

        match rungs.len() {
            0 => Err(ParseError::UnexpectedEnd),
            1 => Ok(rungs[0].clone()),
            _ => Ok(TypeTerm::Ladder(rungs)),
        }
    }

    pub fn parse<It>( &mut self, tokens: &mut Peekable<LadderTypeLexer<It>> ) -> Result<TypeTerm, ParseError>
    where It: Iterator<Item = char>
    {
        match self.parse_partial(tokens) {
            Ok(t) => {
                if let Some(tok) = tokens.peek() {
                    Err(ParseError::UnexpectedToken)
                } else {
                    Ok(t)
                }
            }
            Err(err) => Err(err)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

