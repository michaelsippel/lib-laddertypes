use crate::{sugar::SugaredTypeTerm, unification_sugared, SugaredEnumVariant, SugaredStructMember};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub fn splice_ladders( mut upper: Vec< SugaredTypeTerm >, mut lower: Vec< SugaredTypeTerm >  ) -> Vec< SugaredTypeTerm > {
    eprintln!("splice ladders {:?} <<<====>>>  {:?} ", upper, lower);
    // check for overlap
    for i in 0 .. upper.len() {
        if upper[i] == lower[0] {
            let mut result_ladder = Vec::<SugaredTypeTerm>::new();
            result_ladder.append(&mut upper[0..i].iter().cloned().collect());
            result_ladder.append(&mut lower);
            return result_ladder;
        }
    }

    // no overlap found, just concatenate ladders
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
    pub fn normalize(mut self) -> Self {
        match self {
            SugaredTypeTerm::Ladder(mut rungs) => {
                if rungs.len() == 0 {
                    return SugaredTypeTerm::unit();
                } else if rungs.len() == 1 {
                    return rungs.pop().unwrap().normalize();
                }

                let mut new_rungs = Vec::new();
                let mut r2 = rungs.pop().unwrap().strip();
                while let Some(r1) = rungs.pop() {
                    let r1 = r1.strip();
                    match (r1.clone(), r2.clone()) {
                        (SugaredTypeTerm::Seq { seq_repr: seq_repr1, items: items1 },
                         SugaredTypeTerm::Seq { seq_repr: seq_repr2, items: items2 })
                        => {
                            r2 = SugaredTypeTerm::Seq {
                                    seq_repr:
                                        if seq_repr1.is_some() || seq_repr2.is_some() {
                                            let sr1 = if let Some(seq_repr1) = seq_repr1 { *seq_repr1.clone() }
                                                        else { SugaredTypeTerm::unit() };
                                            let sr2 = if let Some(seq_repr2) = seq_repr2 { *seq_repr2 }
                                                        else { SugaredTypeTerm::unit() };

                                            Some(Box::new(
                                                if sr1 == sr2 {
                                                    sr1
                                                } else if sr1 == SugaredTypeTerm::unit() {
                                                    sr2
                                                } else {
                                                    SugaredTypeTerm::Ladder(vec![ sr1, sr2 ]).normalize()
                                                }))
                                        } else {
                                            None
                                        },
                                    items:
                                        items1.into_iter()
                                            .zip(items2.into_iter())
                                            .map(|(item1, item2)| {
                                                if item1 == item2 {
                                                    item1
                                                } else {
                                                    SugaredTypeTerm::Ladder(vec![ item1.clone(), item2 ])
                                                }
                                            })
                                            .collect()
                                };
                        }

                        (SugaredTypeTerm::Seq { seq_repr, items },
                            SugaredTypeTerm::Spec( mut args )
                        ) => {
                            if args.len() == items.len()+1 {
                                r2 = SugaredTypeTerm::Seq {
                                    seq_repr: Some(Box::new(SugaredTypeTerm::Ladder(vec![
                                        if let Some(seq_repr) = seq_repr {
                                            *seq_repr.clone()
                                        } else {
                                            SugaredTypeTerm::unit()
                                        },
                                        args.remove(0)
                                    ]).normalize())),

                                    items: items.into_iter()
                                        .zip(args.into_iter())
                                        .map(|(i1, i2)| {
                                            if i1 == i2 {
                                                i1
                                            } else {
                                                SugaredTypeTerm::Ladder(vec![ i1, i2 ]).normalize()
                                            }
                                        })
                                        .collect()
                                };
                            } else {
                                new_rungs.push(r2);
                                r2 = r1;
                            }
                        }

                        (SugaredTypeTerm::Struct { struct_repr: struct_repr1, members: members1 },
                         SugaredTypeTerm::Struct { struct_repr: struct_repr2, members: members2 }) => {

                            let mut condensed_struct_repr = None;
                            let mut condensed_members = Vec::new();
                            let mut require_break = false;


                            if let Some(struct_repr1) = struct_repr1 {
                                if let Some(struct_repr2) = struct_repr2 {
                                    condensed_struct_repr = Some(Box::new(SugaredTypeTerm::Ladder(
                                        vec![
                                            struct_repr1.as_ref().clone(),
                                            struct_repr2.as_ref().clone()
                                        ]
                                    ).normalize()))
                                } else {
                                    condensed_struct_repr = Some(Box::new(struct_repr1.as_ref().clone()));
                                }
                            } else {
                                condensed_struct_repr = struct_repr2.clone();
                            }

                            for SugaredStructMember{ symbol: symbol2, ty: ty2 } in members2.iter() {
                                let mut found = false;
                                for SugaredStructMember{ symbol: symbol1, ty: ty1 } in members1.iter() {
                                    if symbol2 == symbol1 {
                                        condensed_members.push(SugaredStructMember {
                                            symbol: symbol1.clone(),
                                            ty: SugaredTypeTerm::Ladder(vec![
                                                ty1.clone(),
                                                ty2.clone()
                                            ]).normalize()
                                        });

                                        found = true;
                                        break;
                                    }
                                }

                                if ! found {
                                    require_break = true;
                                }
                            }

                            if require_break {
                                new_rungs.push(r2);
                                r2 = r1;
                            } else {
                                r2 = SugaredTypeTerm::Struct {
                                    struct_repr: condensed_struct_repr,
                                    members: condensed_members
                                };
                            }
                        }

                        (SugaredTypeTerm::Enum { enum_repr: enum_repr1, variants: variants1 },
                         SugaredTypeTerm::Enum { enum_repr: enum_repr2, variants: variants2 }) => {
                            let mut condensed_enum_repr = None;
                            let mut condensed_variants = Vec::new();
                            let mut require_break = false;

                            if let Some(enum_repr1) = enum_repr1 {
                                if let Some(enum_repr2) = enum_repr2 {
                                    condensed_enum_repr = Some(Box::new(SugaredTypeTerm::Ladder(
                                        vec![
                                            enum_repr1.as_ref().clone(),
                                            enum_repr2.as_ref().clone()
                                        ]
                                    ).normalize()))
                                } else {
                                    condensed_enum_repr = Some(Box::new(enum_repr1.as_ref().clone()));
                                }
                            } else {
                                condensed_enum_repr = enum_repr2.clone();
                            }

                            for SugaredEnumVariant{ symbol: symbol2, ty: ty2 } in variants2.iter() {
                                let mut found = false;
                                for SugaredEnumVariant{ symbol: symbol1, ty: ty1 } in variants1.iter() {
                                    if symbol2 == symbol1 {
                                        condensed_variants.push(SugaredEnumVariant {
                                            symbol: symbol1.clone(),
                                            ty: SugaredTypeTerm::Ladder(vec![
                                                ty1.clone(),
                                                ty2.clone()
                                            ]).normalize()
                                        });

                                        found = true;
                                        break;
                                    }
                                }

                                if ! found {
                                    require_break = true;
                                }
                            }

                            if require_break {
                                new_rungs.push(r2);
                                r2 = r1;
                            } else {
                                r2 = SugaredTypeTerm::Enum {
                                    enum_repr: condensed_enum_repr,
                                    variants: condensed_variants
                                };
                            }
                        }

                        (SugaredTypeTerm::Spec(args1), SugaredTypeTerm::Spec(args2)) => {
                            if args1.len() == args2.len() {
                                if let Ok((ψ,σ)) = unification_sugared::subtype_unify(&args1[0], &args2[0]) {
                                    let mut new_args = Vec::new();

                                    for (a1, a2) in args1.into_iter().zip(args2.into_iter()) {
                                        new_args.push(SugaredTypeTerm::Ladder(vec![ a1, a2 ]).normalize());
                                    }

                                    r2 = SugaredTypeTerm::Spec(new_args);
                                    //new_rungs.push(r2.clone());
                                } else {
                                    new_rungs.push(r2);
                                    r2 = r1;
                                }
                            } else {
                                new_rungs.push(r2);
                                r2 = r1;
                            }
                        }

                        (SugaredTypeTerm::Univ(args1), SugaredTypeTerm::Univ(args2)) => {
                            todo!();
                        }

                        (SugaredTypeTerm::Func(args1), SugaredTypeTerm::Func(args2)) => {
                            todo!();
                        }

                        (SugaredTypeTerm::Morph(args1), SugaredTypeTerm::Morph(args2)) => {
                            todo!();
                        }

                        (SugaredTypeTerm::Ladder(rr1), SugaredTypeTerm::Ladder(rr2)) => {
                            if rr1.len() > 0 {
                                let l = splice_ladders(rr1, rr2);
                                r2 = SugaredTypeTerm::Ladder(l).normalize();
                            }
                        }

                        (atomic1, SugaredTypeTerm::Ladder(mut rr2)) => {
                            if !atomic1.is_empty() {
                                if rr2.first() != Some(&atomic1) {
                                    rr2.insert(0, atomic1);
                                }
                            }
                            r2 = SugaredTypeTerm::Ladder(rr2).normalize();
                        }


                        (SugaredTypeTerm::Ladder(mut rr1), atomic2) => {
                            if !atomic2.is_empty() {
                                if rr1.last() != Some(&atomic2) {
                                    rr1.push(atomic2);
                                }
                            }
                            r2 = SugaredTypeTerm::Ladder(rr1).normalize();
                        }


                        (atomic1, atomic2) => {
                            if atomic1.is_empty() {
                            } else if atomic1 == atomic2 {
                            } else if atomic2.is_empty() {
                                r2 = atomic1;
                            } else {
                                new_rungs.push(atomic2);
                                r2 = atomic1;
                            }
                        }
                    }
                }

                if new_rungs.len() > 0 {
                    new_rungs.push(r2);
                    new_rungs.reverse();
                    return SugaredTypeTerm::Ladder(new_rungs);
                } else {
                    return r2;
                }
            }

            SugaredTypeTerm::Spec(params) => {
                SugaredTypeTerm::Spec(
                    params.into_iter()
                        .map(|p| p.normalize())
                        .collect())
            }

            SugaredTypeTerm::Seq { seq_repr, items } => SugaredTypeTerm::Seq {
                seq_repr: if let Some(seq_repr) = seq_repr { Some(Box::new(seq_repr.normalize())) } else { None },
                items: items.into_iter().map(|p| p.normalize()).collect()
            },
            SugaredTypeTerm::Struct { struct_repr, members } => SugaredTypeTerm::Struct {
                struct_repr: if let Some(struct_repr) = struct_repr { Some(Box::new(struct_repr.normalize())) } else { None },
                members: members.into_iter()
                    .map(|SugaredStructMember{symbol, ty}|
                        SugaredStructMember{ symbol, ty: ty.normalize() })
                    .collect()
            },
            SugaredTypeTerm::Enum { enum_repr, variants } => SugaredTypeTerm::Enum{
                enum_repr: if let Some(enum_repr) = enum_repr { Some(Box::new(enum_repr.normalize())) } else { None },
                variants: variants.into_iter()
                    .map(|SugaredEnumVariant{symbol, ty}|
                        SugaredEnumVariant{ symbol, ty: ty.normalize() })
                    .collect()
            },

            atomic => atomic
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
