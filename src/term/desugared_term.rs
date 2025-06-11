use crate::TypeID;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DesugaredTypeTerm {

    /* Atomic Terms */

    // Base types from dictionary
    TypeID(TypeID),

    // Literals
    Num(i64),
    Char(char),

    /* Complex Terms */

    // Type Parameters
    // avoid currying to save space & indirection
    App(Vec< DesugaredTypeTerm >),

    // Type Ladders
    Ladder(Vec< DesugaredTypeTerm >),
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl DesugaredTypeTerm {
    pub fn unit() -> Self {
        DesugaredTypeTerm::Ladder(vec![])
    }

    pub fn new(id: TypeID) -> Self {
        DesugaredTypeTerm::TypeID(id)
    }

    pub fn arg(&mut self, t: impl Into<DesugaredTypeTerm>) -> &mut Self {
        match self {
            DesugaredTypeTerm::App(args) => {
                args.push(t.into());
            }

            _ => {
                *self = DesugaredTypeTerm::App(vec![
                    self.clone(),
                    t.into()
                ])
            }
        }
        self
    }

    pub fn repr_as(&mut self, t: impl Into<DesugaredTypeTerm>) -> &mut Self {
        match self {
            DesugaredTypeTerm::Ladder(rungs) => {
                rungs.push(t.into());
            }

            _ => {
                *self = DesugaredTypeTerm::Ladder(vec![
                    self.clone(),
                    t.into()
                ])
            }
        }

        self
    }

    pub fn num_arg(&mut self, v: i64) -> &mut Self {
        self.arg(DesugaredTypeTerm::Num(v))
    }

    pub fn char_arg(&mut self, c: char) -> &mut Self {
        self.arg(DesugaredTypeTerm::Char(c))
    }

    pub fn contains_var(&self, var_id: u64) -> bool {
        match self {
            DesugaredTypeTerm::TypeID(TypeID::Var(v)) => &var_id == v,
            DesugaredTypeTerm::App(args) |
            DesugaredTypeTerm::Ladder(args) => {
                for a in args.iter() {
                    if a.contains_var(var_id) {
                        return true;
                    }
                }
                false
            }
            _ => false
        }
    }


    /* strip away empty ladders
     * & unwrap singletons
     */
    pub fn strip(self) -> Self {
        match self {
            DesugaredTypeTerm::Ladder(rungs) => {
                let mut rungs :Vec<_> = rungs.into_iter()
                    .filter_map(|mut r| {
                        r = r.strip();
                        if r != DesugaredTypeTerm::unit() {
                            Some(match r {
                                DesugaredTypeTerm::Ladder(r) => r,
                                a => vec![ a ]
                            })
                        }
                        else { None }
                    })
                .flatten()
                .collect();

                if rungs.len() == 1 {
                    rungs.pop().unwrap()
                } else {
                    DesugaredTypeTerm::Ladder(rungs)
                }
            },
            DesugaredTypeTerm::App(args) => {
                let mut args :Vec<_> = args.into_iter().map(|arg| arg.strip()).collect();
                if args.len() == 0 {
                    DesugaredTypeTerm::unit()
                } else if args.len() == 1 {
                    args.pop().unwrap()
                } else {
                    DesugaredTypeTerm::App(args)
                }
            }
            atom => atom
        }
    }



    pub fn get_interface_type(&self) -> DesugaredTypeTerm {
        match self {
            DesugaredTypeTerm::Ladder(rungs) => {
                if let Some(top) = rungs.first() {
                    top.get_interface_type()
                } else {
                    DesugaredTypeTerm::unit()
                }
            }
            DesugaredTypeTerm::App(args) => {
                DesugaredTypeTerm::App(args.iter().map(|a| a.get_interface_type()).collect())
            }
            atom => atom.clone()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
