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

    pub fn eval_subtype(&mut self, lhs: TypeTerm, rhs: TypeTerm, addr: Vec<usize>) -> Result<(), UnificationError> {
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
                if a1 == a2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(UnificationError{ addr, t1: lhs, t2: rhs}) }
            }

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) => {
                eprintln!("unification: check two ladders");
                for i in 0..a1.len() {
                    if let Some((_, _)) = a1[i].is_semantic_subtype_of( &a2[0] ) {
                        for j in 0..a2.len() {
                            if i+j < a1.len() {
                                let mut new_addr = addr.clone();
                                new_addr.push(i+j);
                                self.eqs.push((a1[i+j].clone(), a2[j].clone(), new_addr))
                            }
                        }
                        return Ok(())
                    }
                }

                Err(UnificationError{ addr, t1: lhs, t2: rhs })
            },

            (t, TypeTerm::Ladder(a1)) => {
                if let Some((idx, τ)) = TypeTerm::Ladder(a1.clone()).is_semantic_subtype_of(&t) {
                    Ok(())
                } else {
                    Err(UnificationError{ addr, t1: TypeTerm::Ladder(a1), t2: t })
                }
            }

            (TypeTerm::Ladder(a1), t) => {
                if let Some((idx, τ)) = TypeTerm::Ladder(a1.clone()).is_semantic_subtype_of(&t) {
                    Ok(())
                } else {
                    Err(UnificationError{ addr, t1: TypeTerm::Ladder(a1), t2: t })
                }
            }

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
                self.σ = new_σ;

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


    pub fn solve_subtype(mut self) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
        while let Some( (mut lhs,mut rhs,addr) ) = self.eqs.pop() {
            lhs.apply_substitution(&|v| self.σ.get(v).cloned());
            rhs.apply_substitution(&|v| self.σ.get(v).cloned());
            eprintln!("eval subtype LHS={:?} || RHS={:?}", lhs, rhs);
            self.eval_subtype(lhs, rhs, addr)?;
        }
        Ok(self.σ)
    }
}

pub fn unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
    let mut unification = UnificationProblem::new(vec![ (t1.clone(), t2.clone()) ]);
    unification.solve()
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
