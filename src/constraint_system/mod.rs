use {
    crate::{dict::*, term::TypeTerm, desugared_term::*, EnumVariant, StructMember, Substitution}, std::collections::HashMap
};

pub mod eval_eq;
pub mod eval_sub;
pub mod eval_trait;
pub mod eval_parallel;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ConstraintError {
    pub addr: Vec<usize>,
    pub t1: TypeTerm,
    pub t2: TypeTerm
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ConstraintPair {
    pub addr: Vec<usize>,
    pub lhs: TypeTerm,
    pub rhs: TypeTerm,
}

impl ConstraintPair {
    pub fn new(lhs: TypeTerm, rhs: TypeTerm) -> Self {
        ConstraintPair {
            lhs,rhs, addr:vec![]
        }
    }
}

pub struct ConstraintSystem {
    σ: HashMap<TypeID, TypeTerm>,
    upper_bounds: HashMap< u64, TypeTerm >,
    lower_bounds: HashMap< u64, TypeTerm >,

    equal_pairs: Vec<ConstraintPair>,
    subtype_pairs: Vec<ConstraintPair>,
    trait_pairs: Vec<ConstraintPair>,
    parallel_pairs: Vec<ConstraintPair>
}

impl ConstraintSystem {
    pub fn new(
        equal_pairs: Vec<ConstraintPair>,
        subtype_pairs: Vec<ConstraintPair>,
        trait_pairs: Vec<ConstraintPair>,
        parallel_pairs: Vec<ConstraintPair>
    ) -> Self {
        ConstraintSystem {
            σ: HashMap::new(),

            equal_pairs,
            subtype_pairs,
            trait_pairs,
            parallel_pairs,

            upper_bounds: HashMap::new(),
            lower_bounds: HashMap::new(),
        }
    }

    pub fn new_eq(eqs: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new(  eqs, Vec::new(), Vec::new(), Vec::new() )
    }

    pub fn new_sub( subs: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( Vec::new(), subs, Vec::new(), Vec::new() )
    }

    pub fn new_trait(traits: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( Vec::new(), Vec::new(), traits, Vec::new() )
    }

    pub fn new_parallel( parallels: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new(Vec::new(), Vec::new(), Vec::new(), parallels )
    }


    /// update all values in substitution
    pub fn reapply_subst(&mut self) {
        let mut new_σ = HashMap::new();
        for (v, tt) in self.σ.iter() {
            let mut tt = tt.clone();
            tt.apply_subst(&self.σ);
            //eprintln!("update σ : {:?} --> {:?}", v, tt);
            new_σ.insert(v.clone(), tt.normalize());
        }
        self.σ = new_σ;
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
                        //eprintln!("var already exists. check max. type");
            if let Ok(halo) = self.eval_subtype(
                ConstraintPair {
                    lhs: lower_bound.clone(),
                    rhs: new_lower_bound.clone(),
                    addr: vec![]
                }
            ) {
                //eprintln!("found more general lower bound");
                //eprintln!("set var {}'s lowerbound to {:?}", v, new_lower_bound.clone());
                // generalize variable type to supertype
                self.lower_bounds.insert(v, new_lower_bound);
                Ok(())
            } else if let Ok(halo) = self.eval_subtype(
                ConstraintPair{
                    lhs: new_lower_bound,
                    rhs: lower_bound,
                    addr: vec![]
                }
            ) {
                //eprintln!("OK, is already larger type");
                Ok(())
            } else {
                //eprintln!("violated subtype restriction");
                Err(())
            }
        } else {
                        //eprintln!("set var {}'s lowerbound to {:?}", v, new_lower_bound.clone());
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
                ConstraintPair {
                    lhs: new_upper_bound.clone(),
                    rhs: upper_bound,
                    addr: vec![]
                }
            ) {
                //println!("found a lower upper bound: {} <= {:?}", v, new_upper_bound);
                // found a lower upper bound
                self.upper_bounds.insert(v, new_upper_bound);
                Ok(())
            } else {
                //println!("new upper bound violates subtype restriction");
                Err(())
            }
        } else {
            //eprintln!("set upper bound: {} <= {:?}", v, new_upper_bound);
            self.upper_bounds.insert(v, new_upper_bound);
            Ok(())
        }
    }

    pub fn solve(mut self) -> Result<(Vec<TypeTerm>, HashMap<TypeID, TypeTerm>), ConstraintError> {
        // solve equations
        while let Some( mut equal_pair ) = self.equal_pairs.pop() {
            equal_pair.lhs.apply_subst(&self.σ);
            equal_pair.rhs.apply_subst(&self.σ);

            self.eval_equation(equal_pair)?;
        }

        // solve subtypes
        //eprintln!("------ SOLVE SUBTYPES ---- ");
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

        //eprintln!("------ MAKE HALOS -----");
        let mut halo_types = Vec::new();
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs = subtype_pair.lhs.apply_subst(&self.σ).clone();
            subtype_pair.rhs = subtype_pair.rhs.apply_subst(&self.σ).clone();

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
) -> Result<HashMap<TypeID, TypeTerm>, ConstraintError> {
    let unification = ConstraintSystem::new_eq(vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    Ok(unification.solve()?.1)
}

pub fn subtype_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), ConstraintError> {
    let unification = ConstraintSystem::new_sub(vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

pub fn parallel_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMap<TypeID, TypeTerm>), ConstraintError> {
    let unification = ConstraintSystem::new_parallel(vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
