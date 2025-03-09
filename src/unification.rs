use {
    crate::{dict::*, term::*}, std::{collections::HashMap, env::consts::ARCH}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UnificationError {
    pub addr: Vec<usize>,
    pub t1: TypeTerm,
    pub t2: TypeTerm
}

#[derive(Clone, Debug)]
pub struct UnificationPair {
    addr: Vec<usize>,
    halo: TypeTerm,

    lhs: TypeTerm,
    rhs: TypeTerm,
}

#[derive(Debug)]
pub struct UnificationProblem {
    σ: HashMap<TypeID, TypeTerm>,
    upper_bounds: HashMap< u64, TypeTerm >,
    lower_bounds: HashMap< u64, TypeTerm >,
    equal_pairs: Vec<UnificationPair>,
    subtype_pairs: Vec<UnificationPair>,
    trait_pairs: Vec<UnificationPair>
}

impl UnificationProblem {
    pub fn new(
        equal_pairs: Vec<(TypeTerm, TypeTerm)>,
        subtype_pairs: Vec<(TypeTerm, TypeTerm)>,
        trait_pairs: Vec<(TypeTerm, TypeTerm)>
    ) -> Self {
        UnificationProblem {
            σ: HashMap::new(),

            equal_pairs: equal_pairs.into_iter().map(|(lhs,rhs)|
                UnificationPair{
                    lhs,rhs,
                    halo: TypeTerm::unit(),
                    addr: Vec::new()
                }).collect(),

            subtype_pairs: subtype_pairs.into_iter().map(|(lhs,rhs)|
                UnificationPair{
                    lhs,rhs,
                    halo: TypeTerm::unit(),
                    addr: Vec::new()
                }).collect(),

            trait_pairs: trait_pairs.into_iter().map(|(lhs,rhs)|
                UnificationPair{
                    lhs,rhs,
                    halo: TypeTerm::unit(),
                    addr: Vec::new()
                }).collect(),

            upper_bounds: HashMap::new(),
            lower_bounds: HashMap::new(),
        }
    }

    pub fn new_eq(eqs: Vec<(TypeTerm, TypeTerm)>) -> Self {
        UnificationProblem::new( eqs, Vec::new(), Vec::new() )
    }

    pub fn new_sub(subs: Vec<(TypeTerm, TypeTerm)>) -> Self {
        UnificationProblem::new( Vec::new(), subs, Vec::new() )
    }


    /// update all values in substitution
    pub fn reapply_subst(&mut self) {
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


    pub fn eval_equation(&mut self, unification_pair: UnificationPair) -> Result<(), UnificationError> {
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
                    Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(*varid)), t2: t.clone() })
                }
            }

            (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) |
            (TypeTerm::App(a1), TypeTerm::App(a2)) => {
                if a1.len() == a2.len() {
                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate().rev() {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push(
                            UnificationPair {
                                lhs: x,
                                rhs: y,
                                halo: TypeTerm::unit(),
                                addr: new_addr
                            });
                    }
                    Ok(())
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }

    pub fn eval_subtype(&mut self, unification_pair: UnificationPair) -> Result<
        // ok: halo type
        TypeTerm,
        // error
        UnificationError
    > {
        match (unification_pair.lhs.clone(), unification_pair.rhs.clone()) {

            /*
             Variables
            */

            (t, TypeTerm::TypeID(TypeID::Var(varid))) => {
//                eprintln!("t <= variable");
                if ! t.contains_var( varid ) {
                   // let x = self.σ.get(&TypeID::Var(varid)).cloned();
                    if let Some(lower_bound) = self.lower_bounds.get(&varid).cloned() {
//                        eprintln!("var already exists. check max. type");
                        if let Ok(halo) = self.eval_subtype(
                            UnificationPair {
                                lhs: lower_bound.clone(),
                                rhs: t.clone(),
                                halo: TypeTerm::unit(),
                                addr: vec![]
                            }
                        ) {
//                            eprintln!("found more general lower bound");
//                            eprintln!("set var {}'s lowerbound to {:?}", varid, t.clone());
                            // generalize variable type to supertype
                            self.lower_bounds.insert(varid, t.clone());
                        } else if let Ok(halo) = self.eval_subtype(
                            UnificationPair{
                                lhs: t.clone(),
                                rhs: lower_bound.clone(),
                                halo: TypeTerm::unit(),
                                addr: vec![]
                            }
                        ) {
//                            eprintln!("OK, is already larger type");
                        } else {
//                            eprintln!("violated subtype restriction");
                            return Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(varid)), t2: t });
                        }
                    } else {
//                        eprintln!("set var {}'s lowerbound to {:?}", varid, t.clone());
                        self.lower_bounds.insert(varid, t.clone());
                    }
                    self.reapply_subst();
                    Ok(TypeTerm::unit())
                } else if t == TypeTerm::TypeID(TypeID::Var(varid)) {
                    Ok(TypeTerm::unit())
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(varid)), t2: t })
                }
            }

            (TypeTerm::TypeID(TypeID::Var(varid)), t) => {
//                eprintln!("variable <= t");

                if let Some(upper_bound) = self.upper_bounds.get(&varid).cloned() {
                    if let Ok(_halo) = self.eval_subtype(
                        UnificationPair {
                            lhs: t.clone(),
                            rhs: upper_bound,
                            halo: TypeTerm::unit(),
                            addr: vec![]
                        }
                    ) {
                        // found a lower upper bound
                        self.upper_bounds.insert(varid, t);
                    }
                } else {
                    self.upper_bounds.insert(varid, t);
                }
                Ok(TypeTerm::unit())
            }



            /*
             Atoms
            */

            (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(TypeTerm::unit()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs}) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(TypeTerm::unit()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(TypeTerm::unit()) } else { Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }


            /*
             Ladders
            */

            (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) => {
                let mut halo = Vec::new();
                for i in 0..a1.len() {
                    let mut new_addr = unification_pair.addr.clone();
                    new_addr.push(i);
                    if let Ok(r_halo) = self.eval_subtype( UnificationPair {
                        lhs: a1[i].clone(),
                        rhs: a2[0].clone(),

                        halo: TypeTerm::unit(),
                        addr: new_addr
                    }) {
//                        eprintln!("unified ladders at {}, r_halo = {:?}", i, r_halo);

                        for j in 0..a2.len() {
                            if i+j < a1.len() {
                                let mut new_addr = unification_pair.addr.clone();
                                new_addr.push(i+j);

                                let lhs = a1[i+j].clone();//.apply_substitution(&|k| self.σ.get(k).cloned()).clone();
                                let rhs = a2[j].clone();//.apply_substitution(&|k| self.σ.get(k).cloned()).clone();

                                if let Ok(rung_halo) = self.eval_subtype(
                                    UnificationPair {
                                        lhs: lhs.clone(), rhs: rhs.clone(),
                                        addr: new_addr.clone(),
                                        halo: TypeTerm::unit()
                                    }
                                ) {
                                    if rung_halo != TypeTerm::unit() {
                                        halo.push(rung_halo);
                                    }
                                } else {
                                    return Err(UnificationError{ addr: new_addr, t1: lhs, t2: rhs })
                                }
                            }
                        }

                        return Ok(
                            if halo.len() == 1 {
                                halo.pop().unwrap()
                            } else {
                                TypeTerm::Ladder(halo)
                            });
                    } else {
                        halo.push(a1[i].clone());
                        //eprintln!("could not unify ladders");
                    }
                }

                Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
            },

            (t, TypeTerm::Ladder(a1)) => {
                Err(UnificationError{ addr: unification_pair.addr, t1: t, t2: TypeTerm::Ladder(a1) })
            }

            (TypeTerm::Ladder(mut a1), t) => {
                let mut new_addr = unification_pair.addr.clone();
                new_addr.push( a1.len() -1 );
                if let Ok(halo) = self.eval_subtype(
                    UnificationPair {
                        lhs: a1.pop().unwrap(),
                        rhs: t.clone(),
                        halo: TypeTerm::unit(),
                        addr: new_addr
                    }
                ) {
                    a1.push(halo);
                    if a1.len() == 1 {
                        Ok(a1.pop().unwrap())
                    } else {
                        Ok(TypeTerm::Ladder(a1))
                    }
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::Ladder(a1), t2: t })
                }
            }


            /*
             Application
            */

            (TypeTerm::App(a1), TypeTerm::App(a2)) => {
                if a1.len() == a2.len() {
                    let mut halo_args = Vec::new();
                    let mut n_halos_required = 0;

                    for (i, (mut x, mut y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate() {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);

                        x = x.strip();

//                        eprintln!("before strip: {:?}", y);
                        y = y.strip();
//                        eprintln!("after strip: {:?}", y);
//                        eprintln!("APP<> eval {:?} \n ?<=? {:?} ", x, y);

                        if let Ok(halo) = self.eval_subtype(
                            UnificationPair {
                                lhs: x.clone(),
                                rhs: y.clone(),
                                halo: TypeTerm::unit(),
                                addr: new_addr,
                            }
                        ) {
                            if halo == TypeTerm::unit() {
                                let mut y = y.clone();
                                y.apply_substitution(&|k| self.σ.get(k).cloned());
                                y = y.strip();
                                let mut top = y.get_lnf_vec().first().unwrap().clone();
                                halo_args.push(top.clone());
//                                eprintln!("add top {:?}", top);
                            } else {
//                                eprintln!("add halo {:?}", halo);
                                if n_halos_required > 0 {
                                    let x = &mut halo_args[n_halos_required-1];
                                    if let TypeTerm::Ladder(argrs) = x {
                                        let mut a = a2[n_halos_required-1].clone();
                                        a.apply_substitution(&|k| self.σ.get(k).cloned());
                                        a = a.get_lnf_vec().first().unwrap().clone();
                                        argrs.push(a);
                                    } else {
                                        *x = TypeTerm::Ladder(vec![
                                            x.clone(),
                                            a2[n_halos_required-1].clone().get_lnf_vec().first().unwrap().clone()
                                        ]);

                                        x.apply_substitution(&|k| self.σ.get(k).cloned());
                                    }
                                }

                                halo_args.push(halo);
                                n_halos_required += 1;
                            }
                        } else {
                            return Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                        }
                    }

                    if n_halos_required > 0 {
//                        eprintln!("halo args : {:?}", halo_args);
                        Ok(TypeTerm::App(halo_args))
                    } else {
                        Ok(TypeTerm::unit())
                    }
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(UnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }

    pub fn solve(mut self) -> Result<(Vec<TypeTerm>, HashMap<TypeID, TypeTerm>), UnificationError> {
        // solve equations
        while let Some( mut equal_pair ) = self.equal_pairs.pop() {
            equal_pair.lhs.apply_substitution(&|v| self.σ.get(v).cloned());
            equal_pair.rhs.apply_substitution(&|v| self.σ.get(v).cloned());

            self.eval_equation(equal_pair)?;
        }

        // solve subtypes
//        eprintln!("------ SOLVE SUBTYPES ---- ");
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs.apply_substitution(&|v| self.σ.get(v).cloned());
            subtype_pair.rhs.apply_substitution(&|v| self.σ.get(v).cloned());
            let _halo = self.eval_subtype( subtype_pair.clone() )?.strip();
        }

        // add variables from subtype bounds
        for (var_id, t) in self.upper_bounds.iter() {
//            eprintln!("VAR {} upper bound {:?}", var_id, t);
            self.σ.insert(TypeID::Var(*var_id), t.clone().strip());
        }

        for (var_id, t) in self.lower_bounds.iter() {
//            eprintln!("VAR {} lower bound {:?}", var_id, t);
            self.σ.insert(TypeID::Var(*var_id), t.clone().strip());
        }

        self.reapply_subst();

//        eprintln!("------ MAKE HALOS -----");
        let mut halo_types = Vec::new();
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs = subtype_pair.lhs.apply_substitution(&|v| self.σ.get(v).cloned()).clone().strip();
            subtype_pair.rhs = subtype_pair.rhs.apply_substitution(&|v| self.σ.get(v).cloned()).clone().strip();

            let halo = self.eval_subtype( subtype_pair.clone() )?.strip();
            halo_types.push(halo);
        }

        // solve traits
        while let Some( trait_pair ) = self.trait_pairs.pop() {
            unimplemented!();
        }

        Ok((halo_types, self.σ))
    }
}

pub fn unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
    let unification = UnificationProblem::new_eq(vec![ (t1.clone(), t2.clone()) ]);
    Ok(unification.solve()?.1)
}

pub fn subtype_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), UnificationError> {
    let unification = UnificationProblem::new_sub(vec![ (t1.clone(), t2.clone()) ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
