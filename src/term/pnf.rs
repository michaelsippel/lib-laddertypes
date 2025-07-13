use std::ops::Deref;

use crate::{constraint_system, subtype_unify, term::TypeTerm, EnumVariant, StructMember};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub fn splice_ladders( mut upper: Vec< TypeTerm >, mut lower: Vec< TypeTerm >  ) -> Vec< TypeTerm > {
    //eprintln!("splice ladders {:?} <<<====>>>  {:?} ", upper, lower);
    // check for overlap
    if lower.len() > 0 {
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
    }
    upper
}

pub fn overlaps( a: &TypeTerm, b: &TypeTerm ) -> bool {
    match (a,b) {
        (TypeTerm::Ladder(rs1), TypeTerm::Ladder(rs2)) => {
            for i in 0 .. rs1.len() {
                let mut diff = false;
                for j in 0 .. usize::min(rs1.len()-i, rs2.len()) {
                    if rs1[i+j] != rs2[j] {
                        diff = true;
                        break;
                    }
                }

                if !diff {
                    return true;
                }
            }

            false
        }

        (TypeTerm::Ladder(rs1), b) => overlaps(rs1.last().unwrap(),b),
        (a, TypeTerm::Ladder(rs2)) => overlaps(a,rs2.first().unwrap()),

        (TypeTerm::Spec(args1), TypeTerm::Spec(args2)) => {
            if args1 == args2 {
                for (a1,a2) in args1.iter().zip(args2.iter()) {
                    if !overlaps(a1,a2) {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        }

        (TypeTerm::Seq { seq_repr, item }, TypeTerm::Seq { seq_repr:sr2, item:i2 }) => {
            todo!()
        }

        (TypeTerm::Struct { struct_repr, members }, TypeTerm::Struct { struct_repr:sr2, members:m2 }) => {
            todo!()
        }

        (TypeTerm::Enum { enum_repr, variants }, TypeTerm::Enum { enum_repr:er2, variants:v2 }) => {
            todo!()
        }

        (TypeTerm::Univ { Γ, bounds, τ }, TypeTerm::Univ { Γ:Γ2, bounds:b2, τ:τ2 }) => {
            todo!()
        }
        (TypeTerm::Morph(m1c, m1d), TypeTerm::Morph(m2c, m2d)) => {
            todo!()
        }
        (TypeTerm::Func(f1s), TypeTerm::Func(f2s)) => {
            todo!()
        }

        (TypeTerm::Id(id1), TypeTerm::Id(id2)) => id1==id2,
        (TypeTerm::Var(id1), TypeTerm::Var(id2)) => id1==id2,
        (TypeTerm::Num(n1), TypeTerm::Num(n2)) => n1==n2,
        (TypeTerm::Char(c1), TypeTerm::Char(c2)) => c1==c2,

        (_,_) => false
    }
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
                        (TypeTerm::Seq { seq_repr: seq_repr1, item: item1 },
                         TypeTerm::Seq { seq_repr: seq_repr2, item: item2 })
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
                                    item:       if item1.deref() == item2.deref() {
                                        item1.clone()
                                    } else {
                                        Box::new(TypeTerm::Ladder(vec![ item1.deref().clone(), item2.deref().clone() ]).normalize())
                                    }
                                };
                        }

                        (TypeTerm::Seq { seq_repr, item },
                         TypeTerm::Spec( mut args )
                        ) => {
                            if args.len() == 2 {
                                let i1 = args.remove(0);
                                let i2 = args.remove(0);

                                if overlaps(item.deref(), &i2) {
                                    r2 = TypeTerm::Seq {
                                        seq_repr: Some(Box::new(TypeTerm::Ladder(vec![
                                            if let Some(seq_repr) = seq_repr {
                                                *seq_repr.clone()
                                            } else {
                                                TypeTerm::unit()
                                            },
                                            i1.clone()
                                        ]).normalize())),

                                        item: Box::new(TypeTerm::Ladder(vec![ item.deref().clone(), i2 ]).normalize())
                                    };
                                } else {
                                    new_rungs.push(r2);
                                    r2 = r1;
                                }
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

                        (TypeTerm::Univ{ Γ:Γ1, bounds:bs1, τ:τ1 }, TypeTerm::Univ{ Γ:Γ2, bounds:bs2, τ:τ2 }) => {
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

            TypeTerm::Seq { seq_repr, item } => TypeTerm::Seq {
                seq_repr: if let Some(seq_repr) = seq_repr { Some(Box::new(seq_repr.normalize())) } else { None },
                item: Box::new(item.normalize())
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
