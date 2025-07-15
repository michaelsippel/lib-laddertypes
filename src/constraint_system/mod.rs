/*
   lib-laddertypes
   Copyright (C) 2023-2025  Michael Sippel
 <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ConstraintPair {
    ValueOf(TypeTerm, TypeTerm),
    Subtype(TypeTerm, TypeTerm),
    Trait(TypeTerm, TypeTerm),
    Parallel(TypeTerm, TypeTerm)
}

impl ConstraintPair {
    pub fn normalize(&self) -> Self {
        match self {
            ConstraintPair::ValueOf(lhs,rhs) => Self::ValueOf(lhs.clone().normalize(), rhs.clone().normalize()),
            ConstraintPair::Subtype(lhs,rhs) => Self::Subtype(lhs.clone().normalize(), rhs.clone().normalize()),
            ConstraintPair::Trait(lhs,rhs) => Self::Trait(lhs.clone().normalize(), rhs.clone().normalize()),
            ConstraintPair::Parallel(lhs,rhs) => Self::Parallel(lhs.clone().normalize(), rhs.clone().normalize()),
        }
    }

    pub fn apply_subst(&mut self, σ: &impl Substitution) -> &mut Self{
        match self {
            ConstraintPair::ValueOf(lhs, rhs) => {
                lhs.apply_subst(σ);
                rhs.apply_subst(σ);
            },
            ConstraintPair::Subtype(lhs, rhs) => {
                lhs.apply_subst(σ);
                rhs.apply_subst(σ);
            },
            ConstraintPair::Trait(lhs, rhs) => {
                lhs.apply_subst(σ);
                rhs.apply_subst(σ);
            },
            ConstraintPair::Parallel(lhs, rhs) => {
                lhs.apply_subst(σ);
                rhs.apply_subst(σ);
            },
        }
        self
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct ConstraintEntry {
    pub addr: Vec<usize>,
    pub bound: ConstraintPair
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct CP2 {
    pub addr: Vec<usize>,
    pub lhs: TypeTerm,
    pub rhs: TypeTerm,
}

impl CP2 {
    pub fn new(lhs: TypeTerm, rhs: TypeTerm) -> Self {
        CP2 {
            lhs,rhs, addr:vec![]
        }
    }
}

pub struct ConstraintSystem {
    σ: HashMapSubst,
    upper_bounds: HashMap< u64, TypeTerm >,
    lower_bounds: HashMap< u64, TypeTerm >,


    // todo: switch CP2 to ConstraintEntry
    //  ```bounds: Vec<ConstraintEntry>```

    equal_pairs: Vec<CP2>,
    subtype_pairs: Vec<CP2>,
    trait_pairs: Vec<CP2>,
    parallel_pairs: Vec<CP2>
}

impl ConstraintSystem {
    pub fn new(
        equal_pairs: Vec<CP2>,
        subtype_pairs: Vec<CP2>,
        trait_pairs: Vec<CP2>,
        parallel_pairs: Vec<CP2>
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

    pub fn new_eq(eqs: Vec<CP2>) -> Self {
        ConstraintSystem::new( eqs, Vec::new(), Vec::new(), Vec::new() )
    }

    pub fn new_sub(subs: Vec<CP2>) -> Self {
        ConstraintSystem::new( Vec::new(), subs, Vec::new(), Vec::new() )
    }

    pub fn new_trait(traits: Vec<CP2>) -> Self {
        ConstraintSystem::new( Vec::new(), Vec::new(), traits, Vec::new() )
    }

    pub fn new_parallel(parallels: Vec<CP2>) -> Self {
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
    let unification = ConstraintSystem::new_eq( vec![ CP2{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    Ok(unification.solve()?.1)
}

pub fn subtype_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMapSubst), ConstraintError> {
    let unification = ConstraintSystem::new_sub(vec![ CP2{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map(|(halos, σ)| (halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

pub fn parallel_unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<(TypeTerm, HashMapSubst), ConstraintError> {
    let unification = ConstraintSystem::new_parallel(vec![ CP2{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map(|(halos, σ)| (halos.first().cloned().unwrap_or(TypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
