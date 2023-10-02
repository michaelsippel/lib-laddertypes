use crate::term::TypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    /// does the type contain ladders (false) or is it 'flat' (true) ?
    ///
    /// Example:
    /// ```<Seq <Digit 10>>``` is flat, but
    /// ```<Digit 10>~Char``` is not
    pub fn is_flat(&self) -> bool {
        match self {
            TypeTerm::TypeID(_) => true,
            TypeTerm::Num(_) => true,
            TypeTerm::Char(_) => true,
            TypeTerm::App(args) => args.iter().fold(true, |s,x| s && x.is_flat()),
            TypeTerm::Ladder(_) => false
        }
    }

    /// transmute type into Ladder-Normal-Form (LNF)
    ///
    /// Example:
    /// ```ignore
    /// <Seq <Digit 10>~Char>
    /// ⇒ <Seq <Digit 10>>~<Seq Char>
    /// ```
    pub fn normalize(self) -> Self {
        let mut new_ladder = Vec::<TypeTerm>::new();

        match self {
            TypeTerm::Ladder(args) => {
                for x in args.into_iter() {
                    new_ladder.push(x.normalize());
                }
            }

            TypeTerm::App(args) => {
                let mut args_iter = args.into_iter();

                new_ladder.push( TypeTerm::App(vec![]) );

                for arg in args_iter {
                    match arg.normalize() {
                        TypeTerm::Ladder(rungs) => {
                            // duplicate last element for each rung
                            let l = new_ladder.len();
                            for _ in 1..rungs.len() {
                                new_ladder.push( new_ladder.last().unwrap().clone() );
                            }

                            for (i,r) in new_ladder.iter_mut().enumerate() {
                                match r {
                                    TypeTerm::App(al) => {
                                        if i < l {
                                            al.push(rungs[0].clone());
                                        } else {
                                            al.push(rungs[i-l+1].clone());
                                        }
                                    }
                                    _ => unreachable!()
                                }
                            }
                        }
                        mut other => {
                            other = other.normalize();
                            for rung in new_ladder.iter_mut() {
                                match rung {
                                    TypeTerm::App(al) => {
                                        al.push(other.clone());
                                    }
                                    _ => unreachable!()
                                }
                            }
                        }
                    }
                }
            }

            atom => {
                new_ladder.push(atom);
            }
        }

        match new_ladder.len() {
            0 => TypeTerm::unit(),
            1 => new_ladder.into_iter().next().unwrap(),
            _ => TypeTerm::Ladder( new_ladder )
        }
    }
    
    /// transmute type into a `Vec` containing
    /// all rungs of the type in LNF
    ///
    /// Example:
    /// ```<Seq <Digit 10>~Char>``` gives
    /// ```ignore
    /// vec![ <Seq <Digit 10>>,  <Seq Char> ]
    /// ```
    pub fn get_lnf_vec(self) -> Vec<TypeTerm> {
        match self.normalize() {
            TypeTerm::Ladder( v ) => {
                v
            },
            _ => {
                unreachable!();
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
