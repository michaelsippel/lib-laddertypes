use crate::{
    TypeTerm,
    TypeID
};

pub const SEQ_SUGARID : TypeID = TypeID::Fun(0);
pub const ENUM_SUGARID : TypeID = TypeID::Fun(1);
pub const STRUCT_SUGARID : TypeID = TypeID::Fun(2);
pub const SUGARID_LIMIT : u64 = 3;

#[derive(Clone)]
pub enum SugaredTypeTerm { 
    /* Atomic Terms */

    // Base types from dictionary
    TypeID(TypeID),

    // Literals
    Num(i64),
    Char(char),

    

    /* Complex Terms */

    // Type Parameters
    // avoid currying to save space & indirection
    App(Vec< SugaredTypeTerm >),

    // Type Ladders
    Ladder(Vec< SugaredTypeTerm >),


    /* Sugar Terms */
    Seq( Vec<SugaredTypeTerm> ),
    Enum( Vec<SugaredTypeTerm> ),
    Struct( Vec<SugaredTypeTerm> )
}


#[derive(Clone)]
struct TypeAssignment {
    id: Option<String>,
    ty: TypeTerm,
}

#[derive(Clone)]
struct LayoutSugar {
    Seq( Vec<TypeAssignment> ),
    Enum( Vec<TypeAssignment> ),
    Struct( Vec<TypeAssignment> )
}



impl SugaredTypeTerm {
    pub fn desugar(self) -> TypeTerm {
        match self {
            SugaredTypeTerm::TypeID(id) => TypeTerm::TypeID(id),
            SugaredTypeTerm::Num(n) => TypeTerm::Num(n),
            SugaredTypeTerm::Char(c) => TypeTerm::Char(c),
            SugaredTypeTerm::App(params) => TypeTerm::App(params.into_iter().map(|p| p.desugar()).collect()),
            SugaredTypeTerm::Ladder(rungs) => TypeTerm::Ladder(rungs.into_iter().map(|p| p.desugar()).collect()),
            SugaredTypeTerm::Seq(pattern) => {
                let mut params : Vec<TypeTerm> = pattern.into_iter().map(|p| p.desugar()).collect();
                params.insert(0, TypeTerm::TypeID(SEQ_SUGARID));
                TypeTerm::App(params)
            }
            SugaredTypeTerm::Enum(options) => {
                let mut params : Vec<TypeTerm> = options.into_iter().map(|o| o.desugar()).collect();
                params.insert(0, TypeTerm::TypeID(ENUM_SUGARID));
                TypeTerm::App(params)
            }
            SugaredTypeTerm::Struct(members) => {
                let mut params : Vec<TypeTerm> = members.into_iter().map(|m| m.desugar()).collect();
                params.insert(0, TypeTerm::TypeID(STRUCT_SUGARID));
                TypeTerm::App(params)
            }
        }
    }

    // normalize any unsugared parts into sugared form
    pub fn sugar(self) -> SugaredTypeTerm {
        // todo: optimize
        self.desugar().sugar()
    }
}

impl TypeTerm {
    pub fn sugar(self) -> SugaredTypeTerm {
        match self {
            TypeTerm::TypeID(id) => SugaredTypeTerm::TypeID(id),
            TypeTerm::Num(n) => SugaredTypeTerm::Num(n),
            TypeTerm::Char(c) => SugaredTypeTerm::Char(c),
            TypeTerm::App(mut params) => {
                if params.len() > 0 {
                    let prim = params.remove(0);
                    match prim {
                        TypeTerm::TypeID( SEQ_SUGARID ) =>
                            SugaredTypeTerm::Seq(
                                params
                                    .into_iter()
                                    .map(|p| p.sugar())
                                    .collect()
                            ),

                        TypeTerm::TypeID( ENUM_SUGARID ) =>
                            SugaredTypeTerm::Enum(
                                params
                                    .into_iter()
                                    .map(|p| p.sugar())
                                    .collect()
                            ),
                        TypeTerm::TypeID( STRUCT_SUGARID ) =>
                            SugaredTypeTerm::Struct(
                                params
                                    .into_iter()
                                    .map(|p| p.sugar())
                                    .collect()
                            ),
                        other => SugaredTypeTerm::App(
                            params.into_iter().map(|p| p.sugar()).collect()
                        )
                    }
                } else {
                    SugaredTypeTerm::App(vec![])
                }
            },
            TypeTerm::Ladder(rungs) => {
                SugaredTypeTerm::Ladder(rungs.into_iter().map(|r| r.sugar()).collect())
            }
        }
    }
}
