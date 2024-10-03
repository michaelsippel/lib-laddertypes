use {
    crate::{TypeTerm, TypeID, parser::ParseLadderType}
};

#[derive(Clone)]
pub enum SugaredTypeTerm {
    TypeID(TypeID),
    Num(i64),
    Char(char),
    Univ(Box< SugaredTypeTerm >),
    Spec(Vec< SugaredTypeTerm >),
    Func(Vec< SugaredTypeTerm >),
    Morph(Vec< SugaredTypeTerm >),
    Ladder(Vec< SugaredTypeTerm >),
    Struct(Vec< SugaredTypeTerm >),
    Enum(Vec< SugaredTypeTerm >),
    Seq(Vec< SugaredTypeTerm >)
}

impl TypeTerm {
    pub fn sugar(self: TypeTerm, dict: &mut impl crate::TypeDict) -> SugaredTypeTerm {
        match self {
            TypeTerm::TypeID(id) => SugaredTypeTerm::TypeID(id),
            TypeTerm::Num(n) => SugaredTypeTerm::Num(n),
            TypeTerm::Char(c) => SugaredTypeTerm::Char(c),
            TypeTerm::App(args) => if let Some(first) = args.first() {
                if first == &dict.parse("Func").unwrap() {
                    SugaredTypeTerm::Func( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Morph").unwrap() {
                    SugaredTypeTerm::Morph( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Struct").unwrap() {
                    SugaredTypeTerm::Struct( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Enum").unwrap() {
                    SugaredTypeTerm::Enum( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Seq").unwrap() {
                    SugaredTypeTerm::Seq( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Spec").unwrap() {
                    SugaredTypeTerm::Spec( args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect() )
                }
                else if first == &dict.parse("Univ").unwrap() {
                    SugaredTypeTerm::Univ(Box::new(
                        SugaredTypeTerm::Spec(
                            args[1..].into_iter().map(|t| t.clone().sugar(dict)).collect()
                        )
                    ))
                }
                else {
                    SugaredTypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
                }
            } else {
                SugaredTypeTerm::Spec(args.into_iter().map(|t| t.sugar(dict)).collect())
            },
            TypeTerm::Ladder(rungs) =>            
               SugaredTypeTerm::Ladder(rungs.into_iter().map(|t| t.sugar(dict)).collect())
        }
    }
}

impl SugaredTypeTerm {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        match self {
            SugaredTypeTerm::TypeID(id) => TypeTerm::TypeID(id),
            SugaredTypeTerm::Num(n) => TypeTerm::Num(n),
            SugaredTypeTerm::Char(c) => TypeTerm::Char(c),
            SugaredTypeTerm::Univ(t) => t.desugar(dict),
            SugaredTypeTerm::Spec(ts) => TypeTerm::App(ts.into_iter().map(|t| t.desugar(dict)).collect()),
            SugaredTypeTerm::Ladder(ts) => TypeTerm::Ladder(ts.into_iter().map(|t|t.desugar(dict)).collect()),
            SugaredTypeTerm::Func(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Func").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Morph(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Morph").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Struct(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Struct").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Enum(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Enum").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
            SugaredTypeTerm::Seq(ts) => TypeTerm::App(
                std::iter::once( dict.parse("Seq").unwrap() ).chain(
                    ts.into_iter().map(|t| t.desugar(dict))
                ).collect()),
        }
    }
}

