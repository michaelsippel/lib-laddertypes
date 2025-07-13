use crate::{term::TypeTerm, constraint_system, EnumVariant, StructMember};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub fn splice_ladders( mut upper: Vec< TypeTerm >, mut lower: Vec< TypeTerm >  ) -> Vec< TypeTerm > {
    eprintln!("splice ladders {:?} <<<====>>>  {:?} ", upper, lower);
    // check for overlap
    for i in 0 .. upper.len() {
        if upper[i] == lower[0] {
            let mut result_ladder = Vec::<TypeTerm>::new();
            result_ladder.append(&mut upper[0..i].iter().cloned().collect());
            result_ladder.append(&mut lower);
            return result_ladder;
        }
    }

    // no overlap found, just concatenate ladders
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
    pub fn normalize(mut self) -> Self {
        match self {
            TypeTerm::Ladder(mut rungs) => {
                if rungs.len() == 0 {
                    return TypeTerm::unit();
                } else if rungs.len() == 1 {
                    return rungs.pop().unwrap().normalize();
                }

                let mut new_rungs = Vec::new();
                let mut r2 = rungs.pop().unwrap().strip();
                while let Some(r1) = rungs.pop() {
                    let r1 = r1.strip();
                    match (r1.clone(), r2.clone()) {
                        (TypeTerm::Seq { seq_repr: seq_repr1, items: items1 },
                         TypeTerm::Seq { seq_repr: seq_repr2, items: items2 })
                        => {
                            r2 = TypeTerm::Seq {
                                    seq_repr:
                                        if seq_repr1.is_some() || seq_repr2.is_some() {
                                            let sr1 = if let Some(seq_repr1) = seq_repr1 { *seq_repr1.clone() }
                                                        else { TypeTerm::unit() };
                                            let sr2 = if let Some(seq_repr2) = seq_repr2 { *seq_repr2 }
                                                        else { TypeTerm::unit() };

                                            Some(Box::new(
                                                if sr1 == sr2 {
                                                    sr1
                                                } else if sr1 == TypeTerm::unit() {
                                                    sr2
                                                } else {
                                                    TypeTerm::Ladder(vec![ sr1, sr2 ]).normalize()
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
                                                    TypeTerm::Ladder(vec![ item1.clone(), item2 ])
                                                }
                                            })
                                            .collect()
                                };
                        }

                        (TypeTerm::Seq { seq_repr, items },
                         TypeTerm::Spec( mut args )
                        ) => {
                            if args.len() == items.len()+1 {
                                r2 = TypeTerm::Seq {
                                    seq_repr: Some(Box::new(TypeTerm::Ladder(vec![
                                        if let Some(seq_repr) = seq_repr {
                                            *seq_repr.clone()
                                        } else {
                                            TypeTerm::unit()
                                        },
                                        args.remove(0)
                                    ]).normalize())),

                                    items: items.into_iter()
                                        .zip(args.into_iter())
                                        .map(|(i1, i2)| {
                                            if i1 == i2 {
                                                i1
                                            } else {
                                                TypeTerm::Ladder(vec![ i1, i2 ]).normalize()
                                            }
                                        })
                                        .collect()
                                };
                            } else {
                                new_rungs.push(r2);
                                r2 = r1;
                            }
                        }

                        (TypeTerm::Struct { struct_repr: struct_repr1, members: members1 },
                         TypeTerm::Struct { struct_repr: struct_repr2, members: members2 }) => {

                            let mut condensed_struct_repr = None;
                            let mut condensed_members = Vec::new();
                            let mut require_break = false;


                            if let Some(struct_repr1) = struct_repr1 {
                                if let Some(struct_repr2) = struct_repr2 {
                                    condensed_struct_repr = Some(Box::new(TypeTerm::Ladder(
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

                            for StructMember{ symbol: symbol2, ty: ty2 } in members2.iter() {
                                let mut found = false;
                                for StructMember{ symbol: symbol1, ty: ty1 } in members1.iter() {
                                    if symbol2 == symbol1 {
                                        condensed_members.push(StructMember {
                                            symbol: symbol1.clone(),
                                            ty: TypeTerm::Ladder(vec![
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
                                r2 = TypeTerm::Struct {
                                    struct_repr: condensed_struct_repr,
                                    members: condensed_members
                                };
                            }
                        }

                        (TypeTerm::Enum { enum_repr: enum_repr1, variants: variants1 },
                         TypeTerm::Enum { enum_repr: enum_repr2, variants: variants2 }) => {
                            let mut condensed_enum_repr = None;
                            let mut condensed_variants = Vec::new();
                            let mut require_break = false;

                            if let Some(enum_repr1) = enum_repr1 {
                                if let Some(enum_repr2) = enum_repr2 {
                                    condensed_enum_repr = Some(Box::new(TypeTerm::Ladder(
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

                            for EnumVariant{ symbol: symbol2, ty: ty2 } in variants2.iter() {
                                let mut found = false;
                                for EnumVariant{ symbol: symbol1, ty: ty1 } in variants1.iter() {
                                    if symbol2 == symbol1 {
                                        condensed_variants.push(EnumVariant {
                                            symbol: symbol1.clone(),
                                            ty: TypeTerm::Ladder(vec![
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
                                r2 = TypeTerm::Enum {
                                    enum_repr: condensed_enum_repr,
                                    variants: condensed_variants
                                };
                            }
                        }

                        (TypeTerm::Spec(args1), TypeTerm::Spec(args2)) => {
                            if args1.len() == args2.len() {
                                if let Ok((ψ,σ)) = constraint_system::subtype_unify(&args1[0], &args2[0]) {
                                    let mut new_args = Vec::new();

                                    for (a1, a2) in args1.into_iter().zip(args2.into_iter()) {
                                        new_args.push(TypeTerm::Ladder(vec![ a1, a2 ]).normalize());
                                    }

                                    r2 = TypeTerm::Spec(new_args);
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

                        (TypeTerm::Univ(bound1, args1), TypeTerm::Univ(bound2, args2)) => {
                            todo!();
                        }

                        (TypeTerm::Func(args1), TypeTerm::Func(args2)) => {
                            todo!();
                        }

                        (TypeTerm::Morph(src1,dst1), TypeTerm::Morph(src2,dst2)) => {
                            todo!();
                        }

                        (TypeTerm::Ladder(rr1), TypeTerm::Ladder(rr2)) => {
                            if rr1.len() > 0 {
                                let l = splice_ladders(rr1, rr2);
                                r2 = TypeTerm::Ladder(l).normalize();
                            }
                        }

                        (atomic1, TypeTerm::Ladder(mut rr2)) => {
                            if !atomic1.is_empty() {
                                if rr2.first() != Some(&atomic1) {
                                    rr2.insert(0, atomic1);
                                }
                            }
                            r2 = TypeTerm::Ladder(rr2).normalize();
                        }


                        (TypeTerm::Ladder(mut rr1), atomic2) => {
                            if !atomic2.is_empty() {
                                if rr1.last() != Some(&atomic2) {
                                    rr1.push(atomic2);
                                }
                            }
                            r2 = TypeTerm::Ladder(rr1).normalize();
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
                    return TypeTerm::Ladder(new_rungs);
                } else {
                    return r2;
                }
            }

            TypeTerm::Spec(params) => {
                TypeTerm::Spec(
                    params.into_iter()
                        .map(|p| p.normalize())
                        .collect())
            }

            TypeTerm::Seq { seq_repr, items } => TypeTerm::Seq {
                seq_repr: if let Some(seq_repr) = seq_repr { Some(Box::new(seq_repr.normalize())) } else { None },
                items: items.into_iter().map(|p| p.normalize()).collect()
            },
            TypeTerm::Struct { struct_repr, members } => TypeTerm::Struct {
                struct_repr: if let Some(struct_repr) = struct_repr { Some(Box::new(struct_repr.normalize())) } else { None },
                members: members.into_iter()
                    .map(|StructMember{symbol, ty}|
                        StructMember{ symbol, ty: ty.normalize() })
                    .collect()
            },
            TypeTerm::Enum { enum_repr, variants } => TypeTerm::Enum {
                enum_repr: if let Some(enum_repr) = enum_repr { Some(Box::new(enum_repr.normalize())) } else { None },
                variants: variants.into_iter()
                    .map(|EnumVariant{symbol, ty}|
                        EnumVariant{ symbol, ty: ty.normalize() })
                    .collect()
            },

            atomic => atomic
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
