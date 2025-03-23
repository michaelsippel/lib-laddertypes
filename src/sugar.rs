use {
    crate::{parser::ParseLadderType, TypeDict, TypeID, TypeTerm}
};

#[derive(Clone, PartialEq)]
pub struct SugaredStructMember {
    pub symbol: String,
    pub ty: SugaredTypeTerm
}

#[derive(Clone, PartialEq)]
pub struct SugaredEnumVariant {
    pub symbol: String,
    pub ty: SugaredTypeTerm
}

#[derive(Clone, PartialEq)]
pub enum SugaredTypeTerm {
    TypeID(TypeID),
    Num(i64),
    Char(char),
    Univ(Box< SugaredTypeTerm >),
    Spec(Vec< SugaredTypeTerm >),
    Func(Vec< SugaredTypeTerm >),
    Morph(Vec< SugaredTypeTerm >),
    Ladder(Vec< SugaredTypeTerm >),
    Struct(Vec< SugaredStructMember >),
    Enum(Vec< SugaredEnumVariant >),
    Seq(Vec< SugaredTypeTerm >)
}

impl SugaredStructMember {
    pub fn parse( dict: &mut impl TypeDict, ty: &TypeTerm ) -> Option<Self> {
        match ty {
            TypeTerm::App(args) => {
                if args.len() != 3 {
                    return None;
                }

                if args[0] != dict.parse("Struct.Field").expect("parse") {
                    return None;
                }

                let symbol = match args[1] {
                    TypeTerm::Char(c) => c.to_string(),
                    TypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[2].clone().sugar(dict);

                Some(SugaredStructMember { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
}

impl SugaredEnumVariant {
    pub fn parse( dict: &mut impl TypeDict, ty: &TypeTerm ) -> Option<Self> {
        match ty {
            TypeTerm::App(args) => {
                if args.len() != 3 {
                    return None;
                }

                if args[0] != dict.parse("Enum.Variant").expect("parse") {
                    return None;
                }

                let symbol = match args[1] {
                    TypeTerm::Char(c) => c.to_string(),
                    TypeTerm::TypeID(id) => dict.get_typename(&id).expect("cant get member name"),
                    _ => {
                        return None;
                    }
                };

                let ty = args[2].clone().sugar(dict);

                Some(SugaredEnumVariant { symbol, ty })
            }
            _ => {
                None
            }
        }
    }
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
                    SugaredTypeTerm::Struct( args[1..].into_iter().map(|t| SugaredStructMember::parse(dict, t).expect("cant parse field")).collect() )
                }
                else if first == &dict.parse("Enum").unwrap() {
                    SugaredTypeTerm::Enum( args[1..].into_iter().map(|t| SugaredEnumVariant::parse(dict, t).expect("cant parse variant")).collect() )
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


impl SugaredStructMember {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        TypeTerm::App(vec![
            dict.parse("Struct.Field").expect("parse"),
            dict.parse(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
    }
}

impl SugaredEnumVariant {
    pub fn desugar(self, dict: &mut impl crate::TypeDict) -> TypeTerm {
        TypeTerm::App(vec![
            dict.parse("Enum.Variant").expect("parse"),
            dict.parse(&self.symbol).expect("parse"),
            self.ty.desugar(dict)
        ])
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

    pub fn is_empty(&self) -> bool {
        match self {
            SugaredTypeTerm::TypeID(_) => false,
            SugaredTypeTerm::Num(_) => false,
            SugaredTypeTerm::Char(_) => false,
            SugaredTypeTerm::Univ(t) => t.is_empty(),
            SugaredTypeTerm::Spec(ts) |
            SugaredTypeTerm::Ladder(ts) |
            SugaredTypeTerm::Func(ts) |
            SugaredTypeTerm::Morph(ts) |
            SugaredTypeTerm::Seq(ts) => {
                ts.iter().fold(true, |s,t| s && t.is_empty() )
            }
            SugaredTypeTerm::Struct(ts) => {
                ts.iter()
                    .fold(true, |s,member_decl| s && member_decl.ty.is_empty() )
            }
            SugaredTypeTerm::Enum(ts) => {
                ts.iter()
                    .fold(true, |s,variant_decl| s && variant_decl.ty.is_empty() )
            }
        }
    }
}
