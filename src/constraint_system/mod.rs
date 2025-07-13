use {
    crate::{
        term::TypeTerm, Substitution,
        context::*,
    },
    std::{collections::HashMap}
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
    σ: HashMapSubst,
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
            σ: HashMapSubst::new(),

            equal_pairs,
            subtype_pairs,
            trait_pairs,
            parallel_pairs,

            upper_bounds: HashMap::new(),
            lower_bounds: HashMap::new(),
        }
    }

    pub fn new_eq(eqs: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( eqs, Vec::new(), Vec::new(), Vec::new() )
    }

    pub fn new_sub(subs: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( Vec::new(), subs, Vec::new(), Vec::new() )
    }

    pub fn new_trait(traits: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( Vec::new(), Vec::new(), traits, Vec::new() )
    }

    pub fn new_parallel(parallels: Vec<ConstraintPair>) -> Self {
        ConstraintSystem::new( Vec::new(), Vec::new(), Vec::new(), parallels )
    }


    /// update all values in substitution
    pub fn reapply_subst(&mut self) {
        self.σ.saturate();
    }

    pub fn solve(mut self) -> Result<(Vec<TypeTerm>, HashMapSubst), ConstraintError> {
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
            self.σ.insert(*var_id, t.clone().strip());
        }

        for (var_id, t) in self.lower_bounds.iter() {
            self.σ.insert(*var_id, t.clone().strip());
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
        while let Some( mut trait_pair ) = self.trait_pairs.pop() {
            trait_pair.lhs.apply_subst(&self.σ);
            trait_pair.rhs.apply_subst(&self.σ);
            self.eval_trait(trait_pair)?;
        }

        Ok((halo_types, self.σ))
    }
}

pub fn unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<HashMapSubst, ConstraintError> {
    let unification = ConstraintSystem::new_eq( vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    Ok(unification.solve()?.1)
}

pub fn subtype_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMapSubst), ConstraintError> {
    let unification = ConstraintSystem::new_sub(vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map(|(halos, σ)| (halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

pub fn parallel_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMapSubst), ConstraintError> {
    let unification = ConstraintSystem::new_parallel(vec![ ConstraintPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map(|(halos, σ)| (halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
