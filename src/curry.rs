use crate::desugared_term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl DesugaredTypeTerm {
    /// transform term to have at max 2 entries in Application list
    pub fn curry(self) -> DesugaredTypeTerm {
        match self {
            DesugaredTypeTerm::App(args) => {
                if args.len() >= 2 {
                    let mut old_args = args.into_iter();
                    let mut new_args = vec![
                        old_args.next().unwrap(),
                        old_args.next().unwrap()
                    ];

                    for x in old_args {
                        new_args = vec![
                            DesugaredTypeTerm::App(new_args),
                            x
                        ];
                    }

                    DesugaredTypeTerm::App(new_args)
                } else {
                    DesugaredTypeTerm::App(args)
                }
            }

            DesugaredTypeTerm::Ladder(rungs) => {
                DesugaredTypeTerm::Ladder(rungs.into_iter().map(|r| r.curry()).collect())
            }

            _ => self
        }
    }

    /// summarize all curried applications into one vec
    pub fn decurry(self) -> Self {
        match self {
            DesugaredTypeTerm::App(mut args) => {
                if args.len() > 0 {
                    let a0 = args.remove(0).decurry();
                    match a0 {
                        DesugaredTypeTerm::App(sub_args) => {
                            for (i,x) in sub_args.into_iter().enumerate() {
                                args.insert(i, x);
                            }
                        }
                        other => { args.insert(0, other); }
                    }
                }
                DesugaredTypeTerm::App(args)
            }
            DesugaredTypeTerm::Ladder(args) => {
                DesugaredTypeTerm::Ladder(args.into_iter().map(|a| a.decurry()).collect())
            }
            _ => self
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
