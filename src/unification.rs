use {
    crate::{dict::*, term::*}, std::{collections::HashMap}
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
    trait_pairs: Vec<UnificationPair>,
    parallel_pairs: Vec<UnificationPair>
}

impl UnificationProblem {
    pub fn new(
        equal_pairs: Vec<(TypeTerm, TypeTerm)>,
        subtype_pairs: Vec<(TypeTerm, TypeTerm)>,
        trait_pairs: Vec<(TypeTerm, TypeTerm)>,
        parallel_pairs: Vec<(TypeTerm, TypeTerm)>
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

            parallel_pairs: parallel_pairs.into_iter().map(|(lhs,rhs)|
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
        UnificationProblem::new( eqs, Vec::new(), Vec::new(), Vec::new() )
    }

    pub fn new_sub(subs: Vec<(TypeTerm, TypeTerm)>) -> Self {
        UnificationProblem::new( Vec::new(), subs, Vec::new(), Vec::new() )
    }


    /// update all values in substitution
    pub fn reapply_subst(&mut self) {
        let mut new_σ = HashMap::new();
        for (v, tt) in self.σ.iter() {
            let mut tt = tt.clone().normalize();
            tt.apply_subst(&self.σ);
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



    pub fn add_lower_subtype_bound(&mut self, v: u64, new_lower_bound: TypeTerm) -> Result<(),()> {

        if new_lower_bound == TypeTerm::TypeID(TypeID::Var(v)) {
            return Ok(());
        }

        if new_lower_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(lower_bound) = self.lower_bounds.get(&v).cloned() {
//                        eprintln!("var already exists. check max. type");
            if let Ok(halo) = self.eval_subtype(
                UnificationPair {
                    lhs: lower_bound.clone(),
                    rhs: new_lower_bound.clone(),
                    halo: TypeTerm::unit(),
                    addr: vec![]
                }
            ) {
//                            eprintln!("found more general lower bound");
//                            eprintln!("set var {}'s lowerbound to {:?}", varid, t.clone());
                // generalize variable type to supertype
                self.lower_bounds.insert(v, new_lower_bound);
                Ok(())
            } else if let Ok(halo) = self.eval_subtype(
                UnificationPair{
                    lhs: new_lower_bound,
                    rhs: lower_bound,
                    halo: TypeTerm::unit(),
                    addr: vec![]
                }
            ) {
//                            eprintln!("OK, is already larger type");
                 Ok(())
            } else {
//                            eprintln!("violated subtype restriction");
                Err(())
            }
        } else {
//                        eprintln!("set var {}'s lowerbound to {:?}", varid, t.clone());
            self.lower_bounds.insert(v, new_lower_bound);
            Ok(())
        }
    }


    pub fn add_upper_subtype_bound(&mut self, v: u64, new_upper_bound: TypeTerm) -> Result<(),()> {
        if new_upper_bound == TypeTerm::TypeID(TypeID::Var(v)) {
            return Ok(());
        }

        if new_upper_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(upper_bound) = self.upper_bounds.get(&v).cloned() {
            if let Ok(_halo) = self.eval_subtype(
                UnificationPair {
                    lhs: new_upper_bound.clone(),
                    rhs: upper_bound,
                    halo: TypeTerm::unit(),
                    addr: vec![]
                }
            ) {
                // found a lower upper bound
                self.upper_bounds.insert(v, new_upper_bound);
                Ok(())
            } else {
                Err(())
            }
        } else {
            self.upper_bounds.insert(v, new_upper_bound);
            Ok(())
        }
    }

    pub fn eval_subtype(&mut self, unification_pair: UnificationPair) -> Result<
        // ok: halo type
        TypeTerm,
        // error
        UnificationError
    > {
        // eprintln!("eval_subtype {:?} <=? {:?}", unification_pair.lhs, unification_pair.rhs);
        match (unification_pair.lhs.clone(), unification_pair.rhs.clone()) {

            /*
             Variables
            */

            (t, TypeTerm::TypeID(TypeID::Var(v))) => {
                //eprintln!("t <= variable");
                if self.add_lower_subtype_bound(v, t.clone()).is_ok() {
                    Ok(TypeTerm::unit())
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(v)), t2: t })
                }
            }

            (TypeTerm::TypeID(TypeID::Var(v)), t) => {
                //eprintln!("variable <= t");
                if self.add_upper_subtype_bound(v, t.clone()).is_ok() {
                    Ok(TypeTerm::unit())
                } else {
                    Err(UnificationError{ addr: unification_pair.addr, t1: TypeTerm::TypeID(TypeID::Var(v)), t2: t })
                }
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
                let mut l1_iter = a1.into_iter().enumerate().rev();
                let mut l2_iter = a2.into_iter().rev();

                let mut halo_ladder = Vec::new();

                while let Some(rhs) = l2_iter.next() {
                    //eprintln!("take rhs = {:?}", rhs);
                    if let Some((i, lhs)) = l1_iter.next() {
                        //eprintln!("take lhs ({}) = {:?}", i, lhs);
                        let mut addr = unification_pair.addr.clone();
                        addr.push(i);
                        //eprintln!("addr = {:?}", addr);

                        match (lhs.clone(), rhs.clone()) {
                            (t, TypeTerm::TypeID(TypeID::Var(v))) => {

                                if self.add_upper_subtype_bound(v,t.clone()).is_ok() {
                                    let mut new_upper_bound_ladder = vec![ t ];

                                    if let Some(next_rhs) = l2_iter.next() {

                                        // TODO

                                    } else {
                                        // take everything

                                        while let Some((i,t)) = l1_iter.next() {
                                            new_upper_bound_ladder.push(t);
                                        }
                                    }

                                    new_upper_bound_ladder.reverse();
                                    if self.add_upper_subtype_bound(v, TypeTerm::Ladder(new_upper_bound_ladder)).is_ok() {
                                        // ok
                                    } else {
                                        return Err(UnificationError {
                                            addr,
                                            t1: lhs,
                                            t2: rhs
                                        });
                                    }
                                } else {
                                    return Err(UnificationError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                            (lhs, rhs) => {
                                if let Ok(ψ) = self.eval_subtype(
                                    UnificationPair {
                                        lhs: lhs.clone(), rhs: rhs.clone(),
                                        addr:addr.clone(), halo: TypeTerm::unit()
                                    }
                                ) {
                                    // ok.
                                    //eprintln!("rungs are subtypes. continue");
                                    halo_ladder.push(ψ);
                                } else {
                                    return Err(UnificationError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                        }
                    } else {
                        // not a subtype,
                        return Err(UnificationError {
                            addr: vec![],
                            t1: unification_pair.lhs,
                            t2: unification_pair.rhs
                        });
                    }
                }
                //eprintln!("left ladder fully consumed");

                for (i,t) in l1_iter {
                    halo_ladder.push(t);
                }
                halo_ladder.reverse();
                Ok(TypeTerm::Ladder(halo_ladder).strip().param_normalize())
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

                        match self.eval_subtype(
                            UnificationPair {
                                lhs: x.clone(),
                                rhs: y.clone(),
                                halo: TypeTerm::unit(),
                                addr: new_addr,
                            }
                        ) {
                            Ok(halo) => {
                            if halo == TypeTerm::unit() {
                                let mut y = y.clone();
                                y.apply_subst(&self.σ);
                                y = y.strip();
                                let mut top = y.get_lnf_vec().first().unwrap().clone();
                                halo_args.push(top.clone());
                                //eprintln!("add top {:?}", top);
                            } else {
                                //eprintln!("add halo {:?}", halo);
                                if n_halos_required > 0 {
                                    let x = &mut halo_args[n_halos_required-1];
                                    if let TypeTerm::Ladder(argrs) = x {
                                        let mut a = a2[n_halos_required-1].clone();
                                        a.apply_subst(&self.σ);
                                        a = a.get_lnf_vec().first().unwrap().clone();
                                        argrs.push(a);
                                    } else {
                                        *x = TypeTerm::Ladder(vec![
                                            x.clone(),
                                            a2[n_halos_required-1].clone().get_lnf_vec().first().unwrap().clone()
                                        ]);

                                        x.apply_subst(&self.σ);
                                    }
                                }

                                halo_args.push(halo);
                                n_halos_required += 1;
                            }
                            },
                            Err(err) => { return Err(err); }
                        }
                    }

                    if n_halos_required > 0 {
                        //eprintln!("halo args : {:?}", halo_args);
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
            equal_pair.lhs.apply_subst(&self.σ);
            equal_pair.rhs.apply_subst(&self.σ);

            self.eval_equation(equal_pair)?;
        }

        // solve subtypes
//        eprintln!("------ SOLVE SUBTYPES ---- ");
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs.apply_subst(&self.σ);
            subtype_pair.rhs.apply_subst(&self.σ);
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
            subtype_pair.lhs = subtype_pair.lhs.apply_subst(&self.σ).clone().strip();
            subtype_pair.rhs = subtype_pair.rhs.apply_subst(&self.σ).clone().strip();

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

pub fn parallel_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), UnificationError> {
    let unification = UnificationProblem::new(vec![], vec![], vec![], vec![ (t1.clone(), t2.clone()) ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
