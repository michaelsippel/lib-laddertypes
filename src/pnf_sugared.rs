use crate::{sugar::SugaredTypeTerm, SugaredEnumVariant, SugaredStructMember};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub fn splice_ladders( mut upper: Vec< SugaredTypeTerm >, mut lower: Vec< SugaredTypeTerm >  ) -> Vec< SugaredTypeTerm > {
    for i in 0 .. upper.len() {
        if upper[i] == lower[0] {
            let mut result_ladder = Vec::<SugaredTypeTerm>::new();
            result_ladder.append(&mut upper[0..i].iter().cloned().collect());
            result_ladder.append(&mut lower);
            return result_ladder;
        }
    }

    upper.append(&mut lower);
    upper
}

impl SugaredTypeTerm {
    /// transmute type into Parameter-Normal-Form (PNF)
    ///
    /// Example:
    /// ```ignore
    /// <Seq <Digit 10>>~<Seq Char>
    /// ⇒ <Seq <Digit 10>~Char>
    /// ```
    pub fn param_normalize(mut self) -> Self {

        match self {
            SugaredTypeTerm::Ladder(mut rungs) => {
                if rungs.len() > 0 {
                    let mut new_rungs = Vec::new();
                    while let Some(bottom) = rungs.pop() {
                        if let Some(last_but) = rungs.last_mut() {
                            match (bottom, last_but) {
                                (SugaredTypeTerm::Spec(bot_args), SugaredTypeTerm::Spec(last_args)) => {
                                    if bot_args.len() == last_args.len() {
                                        let mut new_rung_params = Vec::new();
                                        let mut require_break = false;

                                        if bot_args.len() > 0 {
                                            todo!();
                                            /*
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
                                                            SugaredTypeTerm::Ladder(spliced_type_ladder)
                                                        } else {
                                                            SugaredTypeTerm::unit()
                                                        };

                                                    new_rung_params.push( spliced_type.param_normalize() );
                                                }

                                            } else {
                                                new_rung_params.push(
                                                    SugaredTypeTerm::Ladder(vec![
                                                        last_args[0].clone(),
                                                        bot_args[0].clone()
                                                    ])//.normalize()
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
                                                                SugaredTypeTerm::Ladder(spliced_type_ladder)
                                                            } else {
                                                                SugaredTypeTerm::unit()
                                                            };

                                                        new_rung_params.push( spliced_type.param_normalize() );
                                                    } else {
                                                        new_rung_params.push( bot_args[i].clone() );
                                                        require_break = true;
                                                    }
                                                }
                                            }
                                            */
                                        }

                                        if require_break {
                                            new_rungs.push( SugaredTypeTerm::Spec(new_rung_params) );
                                        } else {
                                            rungs.pop();
                                            rungs.push(SugaredTypeTerm::Spec(new_rung_params));
                                        }

                                    } else {
                                        new_rungs.push( SugaredTypeTerm::Spec(bot_args) );
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
                        SugaredTypeTerm::Ladder(new_rungs)
                    } else if new_rungs.len() == 1 {
                        new_rungs[0].clone()
                    } else {
                        SugaredTypeTerm::unit()
                    }
                } else {
                    SugaredTypeTerm::unit()
                }
            }

            SugaredTypeTerm::Spec(params) => {
                SugaredTypeTerm::Spec(
                    params.into_iter()
                        .map(|p| p.param_normalize())
                        .collect())
            }

            SugaredTypeTerm::Seq { seq_repr, items } => SugaredTypeTerm::Seq {
                seq_repr: if let Some(seq_repr) = seq_repr { Some(Box::new(seq_repr.param_normalize())) } else { None },
                items: items.into_iter().map(|p| p.param_normalize()).collect()
            },
            SugaredTypeTerm::Struct { struct_repr, members } =>SugaredTypeTerm::Struct {
                struct_repr: if let Some(struct_repr) = struct_repr { Some(Box::new(struct_repr.param_normalize())) } else { None },
                members: members.into_iter()
                    .map(|SugaredStructMember{symbol, ty}|
                        SugaredStructMember{ symbol, ty: ty.param_normalize() })
                    .collect()
            },
            SugaredTypeTerm::Enum{ enum_repr, variants } => SugaredTypeTerm::Enum{
                enum_repr: if let Some(enum_repr) = enum_repr { Some(Box::new(enum_repr.param_normalize())) } else { None },
                variants: variants.into_iter()
                    .map(|SugaredEnumVariant{symbol, ty}|
                        SugaredEnumVariant{ symbol, ty: ty.param_normalize() })
                    .collect()
            },

            atomic => atomic
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
