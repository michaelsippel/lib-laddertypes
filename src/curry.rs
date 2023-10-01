use crate::term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    /// transform term to have at max 2 entries in Application list
    pub fn curry(mut self) -> TypeTerm {
        match self {
            TypeTerm::App(args) => {
                if args.len() >= 2 {
                    let mut old_args = args.into_iter();
                    let mut new_args = vec![
                        old_args.next().unwrap(),
                        old_args.next().unwrap()
                    ];

                    for x in old_args {
                        new_args = vec![
                            TypeTerm::App(new_args),
                            x
                        ];
                    }

                    TypeTerm::App(new_args)
                } else {
                    TypeTerm::App(args)
                }
            }

            TypeTerm::Ladder(rungs) => {
                TypeTerm::Ladder(rungs.into_iter().map(|r| r.curry()).collect())
            }

            _ => self
        }
    }

    /// summarize all curried applications into one vec
    pub fn decurry(mut self) -> Self {
        match self {
            TypeTerm::App(mut args) => {
                if args.len() > 0 {
                    let mut a0 = args.remove(0).decurry();
                    match a0 {
                        TypeTerm::App(sub_args) => {
                            for (i,x) in sub_args.into_iter().enumerate() {
                                args.insert(i, x);
                            }
                        }
                        other => { args.insert(0, other); }
                    }
                }
                TypeTerm::App(args)
            }
            TypeTerm::Ladder(args) => {
                TypeTerm::Ladder(args.into_iter().map(|a| a.decurry()).collect())
            }
            _ => self
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
