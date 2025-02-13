use {
    std::collections::HashMap,
    crate::{term::*, dict::*}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UnificationError {
    pub addr: Vec<usize>,
    pub t1: TypeTerm,
    pub t2: TypeTerm
}

pub struct UnificationProblem {
    eqs: Vec<(TypeTerm, TypeTerm, Vec<usize>)>,
    σ: HashMap<TypeID, TypeTerm>
}

impl UnificationProblem {
    pub fn new(eqs: Vec<(TypeTerm, TypeTerm)>) -> Self {
        UnificationProblem {
            eqs: eqs.iter().map(|(lhs,rhs)| (lhs.clone(),rhs.clone(),vec![])).collect(),
            σ: HashMap::new()
        }
    }

    pub fn reapply_subst(&mut self) {
        // update all values in substitution
        let mut new_σ = HashMap::new();
        for (v, tt) in self.σ.iter() {
            let mut tt = tt.clone().normalize();
            tt.apply_substitution(&|v| self.σ.get(v).cloned());
            tt = tt.normalize();
            //eprintln!("update σ : {:?} --> {:?}", v, tt);
            new_σ.insert(v.clone(), tt);
        }
        self.σ = new_σ;
    }

    pub fn eval_subtype(&mut self, lhs: TypeTerm, rhs: TypeTerm, addr: Vec<usize>) -> Result<Vec<TypeTerm>, UnificationError> {
        match (lhs.clone(), rhs.clone()) {
            (TypeTerm::TypeID(TypeID::Var(varid)), t) |
            (t, TypeTerm::TypeID(TypeID::Var(varid))) => {
                self.σ.insert(TypeID::Var(varid), t.clone());

                // update all values in substitution
                let mut new_σ = HashMap::new();
                for (v, tt) in self.σ.iter() {
                    let mut tt = tt.clone().normalize();
                    tt.apply_substitution(&|v| self.σ.get(v).cloned());
                    eprintln!("update σ : {:?} --> {:?}", v, tt);
                    new_σ.insert(v.clone(), tt);
                }
                self.σ = new_σ;

                Ok(())
            }

            (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(vec![]) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(vec![]) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(vec![]) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) => {
                let mut halo = Vec::new();
                for i in 0..a1.len() {
                    if let Ok((r_halo, σ)) = subtype_unify( &a1[i], &a2[0] ) {
                        //eprintln!("unified ladders at {}, r_halo = {:?}", i, r_halo);
                        for (k,v) in σ.iter() {
                            self.σ.insert(k.clone(),v.clone());
                        }

                        for j in 0..a2.len() {
                            if i+j < a1.len() {
                                let mut new_addr = addr.clone();
                                new_addr.push(i+j);
                                self.eqs.push((a1[i+j].clone().apply_substitution(&|k| σ.get(k).cloned()).clone(),
                                    a2[j].clone().apply_substitution(&|k| σ.get(k).cloned()).clone(),
                                    new_addr));
                            }
                        }
                        return Ok(halo)
                    } else {
                        halo.push(a1[i].clone());
                        //eprintln!("could not unify ladders");
                    }
                }

                Err(UnificationError{ addr, t1: lhs, t2: rhs })
            },

            (t, TypeTerm::Ladder(mut a1)) => {
                if let Ok(mut halo) = self.eval_subtype( t.clone(), a1.first().unwrap().clone(), addr.clone() ) {
                    a1.append(&mut halo);
                    Ok(a1)
                } else {
                    Err(UnificationError{ addr, t1: t, t2: TypeTerm::Ladder(a1) })
                }
            }

            (TypeTerm::Ladder(mut a1), t) => {
                if let Ok(mut halo) = self.eval_subtype( a1.pop().unwrap(), t.clone(), addr.clone() ) {
                    a1.append(&mut halo);
                    Ok(a1)
                } else {
                    Err(UnificationError{ addr, t1: TypeTerm::Ladder(a1), t2: t })
                }
            }

            (TypeTerm::App(a1), TypeTerm::App(a2)) => {
                if a1.len() == a2.len() {
                    let mut halo_args = Vec::new();
                    let mut halo_required = false;

                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate() {
                        let mut new_addr = addr.clone();
                        new_addr.push(i);
                        //self.eqs.push((x, y, new_addr));

                        if let Ok(halo) = self.eval_subtype( x.clone(), y.clone(), new_addr ) {
                            if halo.len() == 0 {
                                halo_args.push(y.get_lnf_vec().first().unwrap().clone());
                            } else {
                                halo_args.push(TypeTerm::Ladder(halo));
                                halo_required = true;
                            }
                        } else {
                            return Err(UnificationError{ addr, t1: x, t2: y })
                        }
                    }

                    if halo_required {
                        Ok(vec![ TypeTerm::App(halo_args) ])
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Err(UnificationError{ addr, t1: lhs, t2: rhs })
                }
            }

            _ => Err(UnificationError{ addr, t1: lhs, t2: rhs})
        }
    }

    pub fn eval_equation(&mut self, lhs: TypeTerm, rhs: TypeTerm, addr: Vec<usize>) -> Result<(), UnificationError> {
        match (lhs.clone(), rhs.clone()) {
            (TypeTerm::TypeID(TypeID::Var(varid)), t) |
            (t, TypeTerm::TypeID(TypeID::Var(varid))) => {
                self.σ.insert(TypeID::Var(varid), t.clone());

                // update all values in substitution
                let mut new_σ = HashMap::new();
                for (v, tt) in self.σ.iter() {
                    let mut tt = tt.clone();
                    tt.apply_substitution(&|v| self.σ.get(v).cloned());
                    new_σ.insert(v.clone(), tt);
                }

                self.σ.insert(TypeID::Var(varid), t.clone());
                self.reapply_subst();

                Ok(())
            }

            (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) |
            (TypeTerm::App(a1), TypeTerm::App(a2)) => {
                if a1.len() == a2.len() {
                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate() {
                        let mut new_addr = addr.clone();
                        new_addr.push(i);
                        self.eqs.push((x, y, new_addr));
                    }
                    Ok(())
                } else {
                    Err(UnificationError{ addr, t1: lhs, t2: rhs })
                }
            }

            _ => Err(UnificationError{ addr, t1: lhs, t2: rhs})
        }
    }

    pub fn solve(mut self) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
        while let Some( (mut lhs,mut rhs,addr) ) = self.eqs.pop() {
            lhs.apply_substitution(&|v| self.σ.get(v).cloned());
            rhs.apply_substitution(&|v| self.σ.get(v).cloned());
            self.eval_equation(lhs, rhs, addr)?;
        }
        Ok(self.σ)
    }

    pub fn solve_subtype(mut self) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), UnificationError> {

        pub fn insert_halo_at(
            t: &mut TypeTerm,
            mut addr: Vec<usize>,
            halo_type: TypeTerm
        ) {
            match t {
                TypeTerm::Ladder(rungs) => {
                    if let Some(idx) = addr.pop() {
                        for i in rungs.len()..idx+1 {
                            rungs.push(TypeTerm::unit());
                        }
                        insert_halo_at( &mut rungs[idx], addr, halo_type );
                    } else {
                        rungs.push(halo_type);
                    }
                },

                TypeTerm::App(args) => {
                    if let Some(idx) = addr.pop() {
                        insert_halo_at( &mut args[idx], addr, halo_type );
                    } else {
                        *t = TypeTerm::Ladder(vec![
                            halo_type,
                            t.clone()
                        ]);
                    }
                }

                atomic => {

                }
            }
        }

        //let mut halo_type = TypeTerm::unit();
        let mut halo_rungs = Vec::new();

        while let Some( (mut lhs, mut rhs, mut addr) ) = self.eqs.pop() {
            lhs.apply_substitution(&|v| self.σ.get(v).cloned());
            rhs.apply_substitution(&|v| self.σ.get(v).cloned());
            //eprintln!("eval subtype\n\tLHS={:?}\n\tRHS={:?}\n", lhs, rhs);
            let mut new_halo = self.eval_subtype(lhs, rhs, addr.clone())?;
            if new_halo.len() > 0 {
                //insert_halo_at( &mut halo_type, addr, TypeTerm::Ladder(new_halo) );
                if addr.len() == 0 {
                    halo_rungs.push(TypeTerm::Ladder(new_halo))
                }
            }
        }

        let mut halo_type = TypeTerm::Ladder(halo_rungs);
        halo_type = halo_type.normalize();
        halo_type = halo_type.apply_substitution(&|k| self.σ.get(k).cloned()).clone();

        Ok((halo_type.param_normalize(), self.σ))
}

pub fn unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
    let mut unification = UnificationProblem::new(vec![ (t1.clone(), t2.clone()) ]);
    unification.solve()
}

pub fn subtype_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), UnificationError> {
    let mut unification = UnificationProblem::new(vec![ (t1.clone(), t2.clone()) ]);
    unification.solve_subtype()
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
