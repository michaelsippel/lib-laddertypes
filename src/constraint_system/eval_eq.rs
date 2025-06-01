use {
    crate::{
        dict::*, term::TypeTerm, EnumVariant, StructMember,
        ConstraintSystem, ConstraintPair, ConstraintError
    }
};

impl ConstraintSystem {


    pub fn eval_equation(&mut self, unification_pair: ConstraintPair) -> Result<(), ConstraintError> {
        match (&unification_pair.lhs, &unification_pair.rhs) {
            (TypeTerm::TypeID(TypeID::Var(varid)), t) |
            (t, TypeTerm::TypeID(TypeID::Var(varid))) => {
                if ! t.contains_var( *varid ) {
                    self.σ.insert(TypeID::Var(*varid), t.clone());
                    self.reapply_subst();
                    Ok(())
                } else if t == &TypeTerm::TypeID(TypeID::Var(*varid)) {
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(*varid)), t2: t.clone() })
                }
            }

            (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) |
            (TypeTerm::Spec(a1), TypeTerm::Spec(a2)) => {
                if a1.len() == a2.len() {
                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate().rev() {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push(
                            ConstraintPair {
                                lhs: x,
                                rhs: y,
                                addr: new_addr
                            });
                    }
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            (TypeTerm::Seq{ seq_repr: lhs_seq_repr, items: lhs_items },
                TypeTerm::Seq { seq_repr: rhs_seq_repr, items: rhs_items })
            => {
                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(0);

                if let Some(rhs_seq_repr) = rhs_seq_repr.as_ref() {
                    if let Some(lhs_seq_repr) = lhs_seq_repr.as_ref() {
                        let _seq_repr_ψ = self.eval_equation(ConstraintPair { addr: new_addr.clone(), lhs: *lhs_seq_repr.clone(), rhs: *rhs_seq_repr.clone() })?;
                    } else {
                        return Err(ConstraintError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }


                if lhs_items.len() == rhs_items.len() {
                    for (i, (lhs_ty, rhs_ty)) in lhs_items.into_iter().zip(rhs_items.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( ConstraintPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (TypeTerm::Struct{ struct_repr: lhs_struct_repr, members: lhs_members },
                TypeTerm::Struct{ struct_repr: rhs_struct_repr, members: rhs_members })
            => {
                let new_addr = unification_pair.addr.clone();
                if let Some(rhs_struct_repr) = rhs_struct_repr.as_ref() {
                    if let Some(lhs_struct_repr) = lhs_struct_repr.as_ref() {
                        let _struct_repr_ψ = self.eval_subtype(ConstraintPair { addr: new_addr.clone(), lhs: *lhs_struct_repr.clone(), rhs: *rhs_struct_repr.clone() })?;
                    } else {
                        return Err(ConstraintError{ addr: new_addr.clone(), t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_members.len() == rhs_members.len() {
                    for (i,
                            (StructMember{ symbol: lhs_symbol, ty: lhs_ty},
                                StructMember{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_members.into_iter().zip(rhs_members.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( ConstraintPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (TypeTerm::Enum{ enum_repr: lhs_enum_repr, variants: lhs_variants },
                TypeTerm::Enum{ enum_repr: rhs_enum_repr, variants: rhs_variants })
            => {
                let mut new_addr = unification_pair.addr.clone();
                if let Some(rhs_enum_repr) = rhs_enum_repr.as_ref() {
                    if let Some(lhs_enum_repr) = lhs_enum_repr.as_ref() {
                        let _enum_repr_ψ = self.eval_subtype(ConstraintPair { addr: new_addr.clone(), lhs: *lhs_enum_repr.clone(), rhs: *rhs_enum_repr.clone() })?;
                    } else {
                        return Err(ConstraintError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }


                if lhs_variants.len() == rhs_variants.len() {
                    for (i,
                            (EnumVariant{ symbol: lhs_symbol, ty: lhs_ty },
                                EnumVariant{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_variants.into_iter().zip(rhs_variants.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( ConstraintPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }


}
