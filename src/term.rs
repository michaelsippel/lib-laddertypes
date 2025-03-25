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

    pub fn contains_var(&self, var_id: u64) -> bool {
        match self {
            TypeTerm::TypeID(TypeID::Var(v)) => (&var_id == v),
            TypeTerm::App(args) |
            TypeTerm::Ladder(args) => {
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

    /// recursively apply substitution to all subterms,
    /// which will replace all occurences of variables which map
    /// some type-term in `subst`
    pub fn apply_substitution(
        &mut self,
        subst: &impl Fn(&TypeID) -> Option<TypeTerm>
    ) -> &mut Self {
        match self {
            TypeTerm::TypeID(typid) => {
                if let Some(t) = subst(typid) {
                    *self = t;
                }
            }

            TypeTerm::Ladder(rungs) => {
                for r in rungs.iter_mut() {
                    r.apply_substitution(subst);
                }
            }
            TypeTerm::App(args) => {
                for r in args.iter_mut() {
                    r.apply_substitution(subst);
                }
            }
            _ => {}
        }

        self
    }

    /* strip away empty ladders
     * & unwrap singletons
     */
    pub fn strip(self) -> Self {
        match self {
            TypeTerm::Ladder(rungs) => {
                let mut rungs :Vec<_> = rungs.into_iter()
                    .filter_map(|mut r| {
                        r = r.strip();
                        if r != TypeTerm::unit() {
                            Some(match r {
                                TypeTerm::Ladder(r) => r,
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
                    TypeTerm::Ladder(rungs)
                }
            },
            TypeTerm::App(args) => {
                let mut args :Vec<_> = args.into_iter().map(|arg| arg.strip()).collect();
                if args.len() == 0 {
                    TypeTerm::unit()
                } else if args.len() == 1 {
                    args.pop().unwrap()
                } else {
                    TypeTerm::App(args)
                }
            }
            atom => atom
        }
    }



    pub fn get_interface_type(&self) -> TypeTerm {
        match self {
            TypeTerm::Ladder(rungs) => {
                if let Some(top) = rungs.first() {
                    top.get_interface_type()
                } else {
                    TypeTerm::unit()
                }
            }
            TypeTerm::App(args) => {
                TypeTerm::App(args.iter().map(|a| a.get_interface_type()).collect())
            }
            atom => atom.clone()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
