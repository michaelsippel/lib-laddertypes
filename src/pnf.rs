use crate::term::TypeTerm;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub fn splice_ladders( mut upper: Vec< TypeTerm >, mut lower: Vec< TypeTerm >  ) -> Vec< TypeTerm > {
    for i in 0 .. upper.len() {
        if upper[i] == lower[0] {
            let mut result_ladder = Vec::<TypeTerm>::new();
            result_ladder.append(&mut upper[0..i].iter().cloned().collect());
            result_ladder.append(&mut lower);
            return result_ladder;
        }
    }

    upper.append(&mut lower);
    upper
}

impl TypeTerm {
    /// transmute type into Parameter-Normal-Form (PNF)
    ///
    /// Example:
    /// ```ignore
    /// <Seq <Digit 10>>~<Seq Char>
    /// ⇒ <Seq <Digit 10>~Char>
    /// ```
    pub fn param_normalize(mut self) -> Self {
        match self {
            TypeTerm::Ladder(mut rungs) => {
                if rungs.len() > 0 {
                    let mut new_rungs = Vec::new();
                    while let Some(bottom) = rungs.pop() {
                        if let Some(last_but) = rungs.last_mut() {
                            match (bottom, last_but) {
                                (TypeTerm::App(bot_args), TypeTerm::App(last_args)) => {
                                    if bot_args.len() == last_args.len() {
                                        let mut new_rung_params = Vec::new();
                                        let mut require_break = false;

                                        if bot_args.len() > 0 {
                                            if let Ok(_idx) =  last_args[0].is_syntactic_subtype_of(&bot_args[0]) {
                                                for i in 0 .. bot_args.len() {

                                                    let spliced_type_ladder = splice_ladders(
                                                        last_args[i].clone().get_lnf_vec(),
                                                        bot_args[i].clone().get_lnf_vec()
                                                    );
                                                    let spliced_type =
                                                        if spliced_type_ladder.len() == 1 {
                                                            spliced_type_ladder[0].clone()
                                                        } else if spliced_type_ladder.len() > 1 {
                                                            TypeTerm::Ladder(spliced_type_ladder)
                                                        } else {
                                                            TypeTerm::unit()
                                                        };

                                                    new_rung_params.push( spliced_type.param_normalize() );
                                                }

                                            } else {
                                                new_rung_params.push(
                                                    TypeTerm::Ladder(vec![
                                                        last_args[0].clone(),
                                                        bot_args[0].clone()
                                                    ]).normalize()
                                                );

                                                for i in 1 .. bot_args.len() {
                                                    if let Ok(_idx) = last_args[i].is_syntactic_subtype_of(&bot_args[i]) {
                                                        let spliced_type_ladder = splice_ladders(
                                                            last_args[i].clone().get_lnf_vec(),
                                                            bot_args[i].clone().get_lnf_vec()
                                                        );
                                                        let spliced_type =
                                                            if spliced_type_ladder.len() == 1 {
                                                                spliced_type_ladder[0].clone()
                                                            } else if spliced_type_ladder.len() > 1 {
                                                                TypeTerm::Ladder(spliced_type_ladder)
                                                            } else {
                                                                TypeTerm::unit()
                                                            };

                                                        new_rung_params.push( spliced_type.param_normalize() );
                                                    } else {
                                                        new_rung_params.push( bot_args[i].clone() );
                                                        require_break = true;
                                                    }
                                                }
                                            }
                                        }

                                        if require_break {
                                            new_rungs.push( TypeTerm::App(new_rung_params) );
                                        } else {
                                            rungs.pop();
                                            rungs.push(TypeTerm::App(new_rung_params));
                                        }

                                    } else {
                                        new_rungs.push( TypeTerm::App(bot_args) );
                                    }
                                }
                                (bottom, last_buf) => {
                                    new_rungs.push( bottom );
                                }
                            }
                        } else {
                            new_rungs.push( bottom );
                        }
                    }

                    new_rungs.reverse();

                    if new_rungs.len() > 1 {
                        TypeTerm::Ladder(new_rungs)
                    } else if new_rungs.len() == 1 {
                        new_rungs[0].clone()
                    } else {
                        TypeTerm::unit()
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
