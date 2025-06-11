use {
    crate::{
        term::TypeTerm, ConstraintError, ConstraintPair, ConstraintSystem
    }, std::ops::Deref
};

impl ConstraintSystem {
    /* chek if lhs has trait given by rhs
    */
    pub fn eval_trait(
        &mut self,
        pair: ConstraintPair
    ) -> Result<(), ConstraintError> {
        match (pair.lhs.clone().strip(), pair.rhs.clone().strip()) {

            // check if at least some rung of the ladder has trait τ
            (TypeTerm::Ladder(r1), τ) => {
                for (i, rung) in r1.iter().enumerate() {
                    let mut addr = pair.addr.clone();
                    addr.push(i);

                    if self.eval_trait(ConstraintPair{
                        addr,
                        lhs: rung.clone(),
                        rhs: τ.clone()
                    }).is_ok() {
                        return Ok(());
                    }
                }
                Err(ConstraintError { addr: pair.addr, t1: pair.lhs, t2: pair.rhs })
            }

            // otherwise check for equality

            (TypeTerm::Var(varid), t) |
            (t, TypeTerm::Var(varid)) => {
                if ! t.contains_var( varid ) {
                    self.σ.insert(varid, t.clone());
                    self.reapply_subst();
                    Ok(())
                } else if t == TypeTerm::Var(varid) {
                    Ok(())
                } else {
                    Err(ConstraintError{
                        addr: pair.addr,
                        t1: TypeTerm::Var(varid),
                        t2: t.clone()
                    })
                }
            }

            (TypeTerm::Id(a1), TypeTerm::Id(a2)) => {
                if a1 == a2 { Ok(()) } else { Err(ConstraintError{ addr: pair.addr, t1: pair.lhs, t2: pair.rhs }) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(ConstraintError{ addr: pair.addr, t1: pair.lhs, t2: pair.rhs }) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(ConstraintError{ addr: pair.addr, t1: pair.lhs, t2: pair.rhs }) }
            }

            (TypeTerm::Spec(a1), TypeTerm::Spec(a2)) => {
                if a1.len() == a2.len() {
                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate().rev() {
                        let mut new_addr = pair.addr.clone();
                        new_addr.push(i);
                        self.trait_pairs.push(
                            ConstraintPair {
                                lhs: x,
                                rhs: y,
                                addr: new_addr
                            });
                    }
                    Ok(())
                } else {
                    Err(ConstraintError{ addr: pair.addr, t1: pair.lhs, t2: pair.rhs })
                }
            }

            (TypeTerm::Seq { seq_repr: lhs_sr, items: lhs_it },
                TypeTerm::Seq { seq_repr: rhs_sr, items: rhs_it })
            => {
                {
                    let mut addr = pair.addr.clone();
                    addr.push(0);
                    if let Some(rhs_sr) = rhs_sr {
                        if let Some(lhs_sr) = lhs_sr {
                            self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_sr.deref().clone(), rhs: rhs_sr.deref().clone() });
                        } else {
                            return Err(ConstraintError{ addr, t1: TypeTerm::unit(), t2: rhs_sr.deref().clone() });
                        }
                    }
                }

                for (i, (lhs_member, rhs_member)) in lhs_it.into_iter().zip(rhs_it.into_iter()).enumerate() {
                    let mut addr = pair.addr.clone();
                    addr.push(0);
                    self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_member, rhs: rhs_member });
                }

                Ok(())
            }
            (TypeTerm::Struct { struct_repr: lhs_sr, members: lhs_it },
                TypeTerm::Struct { struct_repr: rhs_sr, members: rhs_it })
            => {
                {
                    let mut addr = pair.addr.clone();
                    addr.push(0);
                    if let Some(rhs_sr) = rhs_sr {
                        if let Some(lhs_sr) = lhs_sr {
                            self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_sr.deref().clone(), rhs: rhs_sr.deref().clone() });
                        } else {
                            return Err(ConstraintError{ addr, t1: TypeTerm::unit(), t2: rhs_sr.deref().clone() });
                        }
                    }
                }

                for (i, rhs_member) in rhs_it.into_iter().enumerate() {
                    let mut found = false;
                    for lhs_member in lhs_it.iter() {
                        if lhs_member.symbol == rhs_member.symbol {
                            let mut addr = pair.addr.clone();
                            addr.push(0);
                            self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_member.ty.clone(), rhs: rhs_member.ty.clone() });
                            found = true;
                            break;
                        }
                    }

                    if ! found {
                        let mut addr = pair.addr.clone();
                        addr.push(i);
                        return Err(ConstraintError { addr, t1: TypeTerm::unit(), t2: rhs_member.ty })
                    }
                }

                Ok(())
            }
            (TypeTerm::Enum { enum_repr: lhs_sr, variants: lhs_it },
                TypeTerm::Enum { enum_repr: rhs_sr, variants: rhs_it })
            => {
                {
                    let mut addr = pair.addr.clone();
                    addr.push(0);
                    if let Some(rhs_sr) = rhs_sr {
                        if let Some(lhs_sr) = lhs_sr {
                            self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_sr.deref().clone(), rhs: rhs_sr.deref().clone() });
                        } else {
                            return Err(ConstraintError{ addr, t1: TypeTerm::unit(), t2: rhs_sr.deref().clone() });
                        }
                    }
                }

                for (i, rhs_member) in rhs_it.into_iter().enumerate() {
                    let mut found = false;
                    for lhs_member in lhs_it.iter() {
                        if lhs_member.symbol == rhs_member.symbol {
                            let mut addr = pair.addr.clone();
                            addr.push(0);
                            self.trait_pairs.push(ConstraintPair { addr, lhs: lhs_member.ty.clone(), rhs: rhs_member.ty.clone() });
                            found = true;
                            break;
                        }
                    }

                    if ! found {
                        let mut addr = pair.addr.clone();
                        addr.push(i);
                        return Err(ConstraintError { addr, t1: TypeTerm::unit(), t2: rhs_member.ty })
                    }
                }

                Ok(())
            }
            _ => Err(ConstraintError { addr: pair.addr, t1: pair.lhs, t2: pair.rhs })
        }
    }

}
