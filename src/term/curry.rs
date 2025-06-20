use crate::term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    /// transform term to have at max 2 entries in Application list
    pub fn curry(self) -> TypeTerm {
        match self {
            TypeTerm::Spec(args) => {
                if args.len() >= 2 {
                    let mut old_args = args.into_iter();
                    let mut new_args = vec![
                        old_args.next().unwrap(),
                        old_args.next().unwrap()
                    ];

                    for x in old_args {
                        new_args = vec![
                            TypeTerm::Spec(new_args),
                            x
                        ];
                    }

                    TypeTerm::Spec(new_args)
                } else {
                    TypeTerm::Spec(args)
                }
            }

            TypeTerm::Ladder(rungs) => {
                TypeTerm::Ladder(rungs.into_iter().map(|r| r.curry()).collect())
            }

            _ => self
        }
    }

    /// summarize all curried applications into one vec
    pub fn decurry(self) -> Self {
        match self {
            TypeTerm::Spec(mut args) => {
                if args.len() > 0 {
                    let a0 = args.remove(0).decurry();
                    match a0 {
                        TypeTerm::Spec(sub_args) => {
                            for (i,x) in sub_args.into_iter().enumerate() {
                                args.insert(i, x);
                            }
                        }
                        other => { args.insert(0, other); }
                    }
                }
                TypeTerm::Spec(args)
            }
            TypeTerm::Ladder(args) => {
                TypeTerm::Ladder(args.into_iter().map(|a| a.decurry()).collect())
            }
            _ => self
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
