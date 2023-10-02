use crate::TypeID;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeTerm {

    /* Atomic Terms */

    // Base types from dictionary
    TypeID(TypeID),

    // Literals
    Num(i64),
    Char(char),

    

    /* Complex Terms */

    // Type Parameters
    // avoid currying to save space & indirection
    App(Vec< TypeTerm >),

    // Type Ladders
    Ladder(Vec< TypeTerm >),
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    pub fn unit() -> Self {
        TypeTerm::Ladder(vec![])
    }

    pub fn new(id: TypeID) -> Self {
        TypeTerm::TypeID(id)
    }

    pub fn arg(&mut self, t: impl Into<TypeTerm>) -> &mut Self {
        match self {
            TypeTerm::App(args) => {
                args.push(t.into());
            }

            _ => {
                *self = TypeTerm::App(vec![
                    self.clone(),
                    t.into()
                ])                
            }
        }

        self
    }

    pub fn repr_as(&mut self, t: impl Into<TypeTerm>) -> &mut Self {
        match self {
            TypeTerm::Ladder(rungs) => {
                rungs.push(t.into());                
            }

            _ => {
                *self = TypeTerm::Ladder(vec![
                    self.clone(),
                    t.into()
                ])
            }
        }

        self
    }

    pub fn num_arg(&mut self, v: i64) -> &mut Self {
        self.arg(TypeTerm::Num(v))
    }

    pub fn char_arg(&mut self, c: char) -> &mut Self {
        self.arg(TypeTerm::Char(c))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
