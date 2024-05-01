use crate::term::TypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    /// transmute type into Parameter-Normal-Form (PNF)
    ///
    /// Example:
    /// ```ignore
    /// <Seq <Digit 10>>~<Seq Char>
    /// ⇒ <Seq <Digit 10>~Char>
    /// ```
    pub fn param_normalize(self) -> Self {
        match self {
            TypeTerm::Ladder(mut rungs) => {
                if rungs.len() > 0 {
                    // normalize all rungs separately
                    for r in rungs.iter_mut() {
                        *r = r.clone().param_normalize();
                    }

                    // take top-rung
                    match rungs.remove(0) {
                        TypeTerm::App(params_top) => {
                            let mut params_ladders = Vec::new();
                            let mut tail : Vec<TypeTerm> = Vec::new();

                            // append all other rungs to ladders inside
                            // the application
                            for p in params_top {
                                params_ladders.push(vec![ p ]);
                            }

                            for r in rungs {
                                match r {
                                    TypeTerm::App(mut params_rung) => {
                                        if params_rung.len() > 0 {
                                            let mut first_param = params_rung.remove(0); 

                                            if first_param == params_ladders[0][0] {
                                               for (l, p) in params_ladders.iter_mut().skip(1).zip(params_rung) {
                                                   l.push(p.param_normalize());
                                               }
                                            } else {
                                               params_rung.insert(0, first_param);
                                               tail.push(TypeTerm::App(params_rung));
                                            }
                                        }
                                    }

                                    TypeTerm::Ladder(mut rs) => {
                                        for r in rs {
                                            tail.push(r.param_normalize());
                                        }
                                    }

                                    atomic => {
                                        tail.push(atomic);
                                    }
                                }
                            }

                            let head = TypeTerm::App(
                                params_ladders.into_iter()
                                    .map(
                                        |mut l| {
                                            l.dedup();
                                            match l.len() {
                                                0 => TypeTerm::unit(),
                                                1 => l.remove(0),
                                                _ => TypeTerm::Ladder(l).param_normalize()
                                            }
                                        }
                                    )
                                    .collect()
                            );

                            if tail.len() > 0 {
                                tail.insert(0, head);
                                TypeTerm::Ladder(tail)
                            } else {
                                head
                            }
                        }

                        TypeTerm::Ladder(mut r) => {
                            r.append(&mut rungs);
                            TypeTerm::Ladder(r)
                        }

                        atomic => {
                            rungs.insert(0, atomic);
                            TypeTerm::Ladder(rungs)
                        }
                    }
                } else {
                    TypeTerm::unit()
                }
            }

            TypeTerm::App(params) => {
                TypeTerm::App(
                    params.into_iter()
                        .map(|p| p.param_normalize())
                        .collect())
            }

            atomic => atomic
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
