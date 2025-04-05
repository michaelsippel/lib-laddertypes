use {
    crate::{dict::*, sugar::SugaredTypeTerm, term::*, SugaredEnumVariant, SugaredStructMember}, std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SugaredUnificationError {
    pub addr: Vec<usize>,
    pub t1: SugaredTypeTerm,
    pub t2: SugaredTypeTerm
}

// todo : rename -> ConstraintPair
#[derive(Clone)]
pub struct SugaredUnificationPair {
    addr: Vec<usize>,
    lhs: SugaredTypeTerm,
    rhs: SugaredTypeTerm,
}

impl SugaredUnificationPair {
    pub fn new(lhs: SugaredTypeTerm, rhs: SugaredTypeTerm) -> Self {
        SugaredUnificationPair {
            lhs,rhs, addr:vec![]
        }
    }
}

// todo : Rename -> ConstraintSystem
pub struct SugaredUnificationProblem {
   // dict: &'a Dict,

    σ: HashMap<TypeID, SugaredTypeTerm>,
    upper_bounds: HashMap< u64, SugaredTypeTerm >,
    lower_bounds: HashMap< u64, SugaredTypeTerm >,

    equal_pairs: Vec<SugaredUnificationPair>,
    subtype_pairs: Vec<SugaredUnificationPair>,
    trait_pairs: Vec<SugaredUnificationPair>,
    parallel_pairs: Vec<SugaredUnificationPair>
}

impl SugaredUnificationProblem {
    pub fn new(
       // dict: &'a mut Dict,
        equal_pairs: Vec<SugaredUnificationPair>,
        subtype_pairs: Vec<SugaredUnificationPair>,
        trait_pairs: Vec<SugaredUnificationPair>,
        parallel_pairs: Vec<SugaredUnificationPair>
    ) -> Self {
        SugaredUnificationProblem {
          //  dict,
            σ: HashMap::new(),

            equal_pairs,
            subtype_pairs,
            trait_pairs,
            parallel_pairs,

            upper_bounds: HashMap::new(),
            lower_bounds: HashMap::new(),
        }
    }

    pub fn new_eq(eqs: Vec<SugaredUnificationPair>) -> Self {
        SugaredUnificationProblem::new(  eqs, Vec::new(), Vec::new(), Vec::new() )
    }

    pub fn new_sub( subs: Vec<SugaredUnificationPair>) -> Self {
        SugaredUnificationProblem::new( Vec::new(), subs, Vec::new(), Vec::new() )
    }

    pub fn new_trait(traits: Vec<SugaredUnificationPair>) -> Self {
        SugaredUnificationProblem::new( Vec::new(), Vec::new(), traits, Vec::new() )
    }

    pub fn new_parallel( parallels: Vec<SugaredUnificationPair>) -> Self {
        SugaredUnificationProblem::new(Vec::new(), Vec::new(), Vec::new(), parallels )
    }


    /// update all values in substitution
    pub fn reapply_subst(&mut self) {
        let mut new_σ = HashMap::new();
        for (v, tt) in self.σ.iter() {
            let mut tt = tt.clone();
            tt.apply_subst(&self.σ);
            //eprintln!("update σ : {:?} --> {:?}", v, tt);
            new_σ.insert(v.clone(), tt);
        }
        self.σ = new_σ;
    }


    pub fn eval_equation(&mut self, unification_pair: SugaredUnificationPair) -> Result<(), SugaredUnificationError> {
        match (&unification_pair.lhs, &unification_pair.rhs) {
            (SugaredTypeTerm::TypeID(TypeID::Var(varid)), t) |
            (t, SugaredTypeTerm::TypeID(TypeID::Var(varid))) => {
                if ! t.contains_var( *varid ) {
                    self.σ.insert(TypeID::Var(*varid), t.clone());
                    self.reapply_subst();
                    Ok(())
                } else if t == &SugaredTypeTerm::TypeID(TypeID::Var(*varid)) {
                    Ok(())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: SugaredTypeTerm::TypeID(TypeID::Var(*varid)), t2: t.clone() })
                }
            }

            (SugaredTypeTerm::TypeID(a1), SugaredTypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (SugaredTypeTerm::Num(n1), SugaredTypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (SugaredTypeTerm::Char(c1), SugaredTypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }

            (SugaredTypeTerm::Ladder(a1), SugaredTypeTerm::Ladder(a2)) |
            (SugaredTypeTerm::Spec(a1), SugaredTypeTerm::Spec(a2)) => {
                if a1.len() == a2.len() {
                    for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate().rev() {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push(
                            SugaredUnificationPair {
                                lhs: x,
                                rhs: y,
                                addr: new_addr
                            });
                    }
                    Ok(())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            (SugaredTypeTerm::Seq{ seq_repr: lhs_seq_repr, items: lhs_items },
                SugaredTypeTerm::Seq { seq_repr: rhs_seq_repr, items: rhs_items })
            => {
                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(0);

                if let Some(rhs_seq_repr) = rhs_seq_repr.as_ref() {
                    if let Some(lhs_seq_repr) = lhs_seq_repr.as_ref() {
                        let _seq_repr_ψ = self.eval_equation(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_seq_repr.clone(), rhs: *rhs_seq_repr.clone() })?;
                    } else {
                        return Err(SugaredUnificationError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }


                if lhs_items.len() == rhs_items.len() {
                    for (i, (lhs_ty, rhs_ty)) in lhs_items.into_iter().zip(rhs_items.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( SugaredUnificationPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (SugaredTypeTerm::Struct{ struct_repr: lhs_struct_repr, members: lhs_members },
                SugaredTypeTerm::Struct{ struct_repr: rhs_struct_repr, members: rhs_members })
            => {
                let new_addr = unification_pair.addr.clone();
                if let Some(rhs_struct_repr) = rhs_struct_repr.as_ref() {
                    if let Some(lhs_struct_repr) = lhs_struct_repr.as_ref() {
                        let _struct_repr_ψ = self.eval_subtype(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_struct_repr.clone(), rhs: *rhs_struct_repr.clone() })?;
                    } else {
                        return Err(SugaredUnificationError{ addr: new_addr.clone(), t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_members.len() == rhs_members.len() {
                    for (i,
                            (SugaredStructMember{ symbol: lhs_symbol, ty: lhs_ty},
                                SugaredStructMember{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_members.into_iter().zip(rhs_members.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( SugaredUnificationPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (SugaredTypeTerm::Enum{ enum_repr: lhs_enum_repr, variants: lhs_variants },
                SugaredTypeTerm::Enum{ enum_repr: rhs_enum_repr, variants: rhs_variants })
            => {
                let mut new_addr = unification_pair.addr.clone();
                if let Some(rhs_enum_repr) = rhs_enum_repr.as_ref() {
                    if let Some(lhs_enum_repr) = lhs_enum_repr.as_ref() {
                        let _enum_repr_ψ = self.eval_subtype(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_enum_repr.clone(), rhs: *rhs_enum_repr.clone() })?;
                    } else {
                        return Err(SugaredUnificationError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }


                if lhs_variants.len() == rhs_variants.len() {
                    for (i,
                            (SugaredEnumVariant{ symbol: lhs_symbol, ty: lhs_ty },
                                SugaredEnumVariant{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_variants.into_iter().zip(rhs_variants.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        self.equal_pairs.push( SugaredUnificationPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } );
                    }
                    Ok(())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }



    pub fn add_lower_subtype_bound(&mut self, v: u64, new_lower_bound: SugaredTypeTerm) -> Result<(),()> {

        if new_lower_bound == SugaredTypeTerm::TypeID(TypeID::Var(v)) {
            return Ok(());
        }

        if new_lower_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(lower_bound) = self.lower_bounds.get(&v).cloned() {
            if let Ok(halo) = self.eval_subtype(
                SugaredUnificationPair {
                    lhs: lower_bound.clone(),
                    rhs: new_lower_bound.clone(),
                    addr: vec![]
                }
            ) {
                // generalize variable type to supertype
                self.lower_bounds.insert(v, new_lower_bound);
                Ok(())
            } else if let Ok(halo) = self.eval_subtype(
                SugaredUnificationPair{
                    lhs: new_lower_bound,
                    rhs: lower_bound,
                    addr: vec![]
                }
            ) {
                 Ok(())
            } else {
                Err(())
            }
        } else {
            self.lower_bounds.insert(v, new_lower_bound);
            Ok(())
        }
    }


    pub fn add_upper_subtype_bound(&mut self, v: u64, new_upper_bound: SugaredTypeTerm) -> Result<(),()> {
        if new_upper_bound == SugaredTypeTerm::TypeID(TypeID::Var(v)) {
            return Ok(());
        }

        if new_upper_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(upper_bound) = self.upper_bounds.get(&v).cloned() {
            if let Ok(_halo) = self.eval_subtype(
                SugaredUnificationPair {
                    lhs: new_upper_bound.clone(),
                    rhs: upper_bound,
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

    pub fn eval_subtype(&mut self, unification_pair: SugaredUnificationPair) -> Result<
        // ok: halo type
        SugaredTypeTerm,
        // error
        SugaredUnificationError
    > {
        match (unification_pair.lhs.clone().strip(), unification_pair.rhs.clone().strip()) {

            /*
             Variables
            */

            (SugaredTypeTerm::TypeID(TypeID::Var(v)), t) => {
                //eprintln!("variable <= t");
                if self.add_upper_subtype_bound(v, t.clone()).is_ok() {
                    Ok(SugaredTypeTerm::unit())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: SugaredTypeTerm::TypeID(TypeID::Var(v)), t2: t })
                }
            }

            (t, SugaredTypeTerm::TypeID(TypeID::Var(v))) => {
                //eprintln!("t <= variable");
                if self.add_lower_subtype_bound(v, t.clone()).is_ok() {
                    Ok(SugaredTypeTerm::unit())
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: SugaredTypeTerm::TypeID(TypeID::Var(v)), t2: t })
                }
            }


            /*
             Atoms
            */
            (SugaredTypeTerm::TypeID(a1), SugaredTypeTerm::TypeID(a2)) => {
                if a1 == a2 { Ok(SugaredTypeTerm::unit()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs}) }
            }
            (SugaredTypeTerm::Num(n1), SugaredTypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(SugaredTypeTerm::unit()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (SugaredTypeTerm::Char(c1), SugaredTypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(SugaredTypeTerm::unit()) } else { Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }

            /*
             Complex Types
            */

            (SugaredTypeTerm::Seq{ seq_repr: lhs_seq_repr, items: lhs_items },
                SugaredTypeTerm::Seq { seq_repr: rhs_seq_repr, items: rhs_items })
            => {
                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(0);

                if let Some(rhs_seq_repr) = rhs_seq_repr.as_ref() {
                    //eprintln!("subtype unify: rhs has seq-repr: {:?}", rhs_seq_repr);
                    if let Some(lhs_seq_repr) = lhs_seq_repr.as_ref() {
                        //eprintln!("check if it maches lhs seq-repr: {:?}", lhs_seq_repr);
                        let _seq_repr_ψ = self.eval_subtype(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_seq_repr.clone(), rhs: *rhs_seq_repr.clone() })?;
                        //eprintln!("..yes!");
                    } else {
                        //eprintln!("...but lhs has none.");
                        return Err(SugaredUnificationError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(1);
                if lhs_items.len() == rhs_items.len() && lhs_items.len() > 0 {
                    match self.eval_subtype( SugaredUnificationPair { addr: new_addr.clone(), lhs: lhs_items[0].clone(), rhs: rhs_items[0].clone() } ) {
                        Ok(ψ) => Ok(SugaredTypeTerm::Seq {
                                seq_repr: None, // <<- todo
                                items: vec![ψ]
                            }.strip()),
                        Err(e) => Err(SugaredUnificationError{
                                addr: new_addr,
                                t1: e.t1,
                                t2: e.t2,
                            })
                    }
                } else {
                    Err(SugaredUnificationError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (SugaredTypeTerm::Struct{ struct_repr: lhs_struct_repr, members: lhs_members },
                SugaredTypeTerm::Struct{ struct_repr: rhs_struct_repr, members: rhs_members })
            => {
                let new_addr = unification_pair.addr.clone();
                if let Some(rhs_struct_repr) = rhs_struct_repr.as_ref() {
                    if let Some(lhs_struct_repr) = lhs_struct_repr.as_ref() {
                        let _struct_repr_ψ = self.eval_subtype(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_struct_repr.clone(), rhs: *rhs_struct_repr.clone() })?;
                    } else {
                        return Err(SugaredUnificationError{ addr: new_addr.clone(), t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_members.len() == rhs_members.len() {
                    let mut halo_members = Vec::new();
                    for (i,
                            (SugaredStructMember{ symbol: lhs_symbol, ty: lhs_ty},
                                SugaredStructMember{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_members.into_iter().zip(rhs_members.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);

                        let ψ = self.eval_subtype( SugaredUnificationPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } )?;
                        halo_members.push(SugaredStructMember { symbol: lhs_symbol, ty: ψ });
                    }
                    Ok(SugaredTypeTerm::Struct {
                        struct_repr: None,
                        members: halo_members
                    })
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }
            (SugaredTypeTerm::Enum{ enum_repr: lhs_enum_repr, variants: lhs_variants },
                SugaredTypeTerm::Enum{ enum_repr: rhs_enum_repr, variants: rhs_variants })
            => {
                let mut new_addr = unification_pair.addr.clone();
                if let Some(rhs_enum_repr) = rhs_enum_repr.as_ref() {
                    if let Some(lhs_enum_repr) = lhs_enum_repr.as_ref() {
                        let _enum_repr_ψ = self.eval_subtype(SugaredUnificationPair { addr: new_addr.clone(), lhs: *lhs_enum_repr.clone(), rhs: *rhs_enum_repr.clone() })?;
                    } else {
                        return Err(SugaredUnificationError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_variants.len() == rhs_variants.len() {
                    let mut halo_variants = Vec::new();
                    for (i,
                            (SugaredEnumVariant{ symbol: lhs_symbol, ty: lhs_ty },
                                SugaredEnumVariant{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_variants.into_iter().zip(rhs_variants.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        let ψ = self.eval_subtype( SugaredUnificationPair { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } )?;
                        halo_variants.push(SugaredEnumVariant { symbol: lhs_symbol, ty: ψ });
                    }
                    Ok(SugaredTypeTerm::Enum {
                        enum_repr: None,
                        variants: halo_variants
                    })
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }



            /*
             Ladders
            */

            (SugaredTypeTerm::Ladder(a1), SugaredTypeTerm::Ladder(a2)) => {

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
                            (t, SugaredTypeTerm::TypeID(TypeID::Var(v))) => {

                                if self.add_upper_subtype_bound(v,t.clone()).is_ok() {
                                    let mut new_upper_bound_ladder = vec![ t ];

                                    if let Some(next_rhs) = l2_iter.next() {

                                        // TODO
                                        todo!();

                                    } else {
                                        // ladder of rhs is empty
                                        // take everything

                                        while let Some((i,t)) = l1_iter.next() {
                                            new_upper_bound_ladder.push(t);
                                        }
                                    }

                                    new_upper_bound_ladder.reverse();
                                    if self.add_upper_subtype_bound(v, SugaredTypeTerm::Ladder(new_upper_bound_ladder)).is_ok() {
                                        // ok
                                    } else {
                                        return Err(SugaredUnificationError {
                                            addr,
                                            t1: lhs,
                                            t2: rhs
                                        });
                                    }
                                } else {
                                    return Err(SugaredUnificationError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                            (lhs, rhs) => {
                                if let Ok(ψ) = self.eval_subtype(
                                    SugaredUnificationPair {
                                        lhs: lhs.clone(),
                                        rhs: rhs.clone(),
                                        addr:addr.clone(),
                                    }
                                ) {
                                    // ok.
                                    //eprintln!("rungs are subtypes. continue");
                                    halo_ladder.push(ψ);
                                } else {
                                    return Err(SugaredUnificationError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                        }
                    } else {
                        // not a subtype,
                        return Err(SugaredUnificationError {
                            addr: vec![],
                            t1: unification_pair.lhs,
                            t2: unification_pair.rhs
                        });
                    }
                }
                //eprintln!("left ladder fully consumed");

                for (i,t) in l1_iter {
                    //!("push {} to halo ladder", t.pretty(self.dict,0));
                    halo_ladder.push(t);
                }
                halo_ladder.reverse();
                Ok(SugaredTypeTerm::Ladder(halo_ladder).strip())//.param_normalize())
            },

            (t, SugaredTypeTerm::Ladder(a1)) => {
                Err(SugaredUnificationError{ addr: unification_pair.addr, t1: t, t2: SugaredTypeTerm::Ladder(a1) })
            }

            (SugaredTypeTerm::Ladder(mut a1), t) => {
                if a1.len() > 0 {
                    let mut new_addr = unification_pair.addr.clone();
                    new_addr.push( a1.len() -1 );
                    if let Ok(halo) = self.eval_subtype(
                        SugaredUnificationPair {
                            lhs: a1.pop().unwrap(),
                            rhs: t.clone(),
                            addr: new_addr
                        }
                    ) {
                        a1.push(halo);
                        if a1.len() == 1 {
                            Ok(a1.pop().unwrap())
                        } else {
                            Ok(SugaredTypeTerm::Ladder(a1))
                        }
                    } else {
                        Err(SugaredUnificationError{ addr: unification_pair.addr, t1: SugaredTypeTerm::Ladder(a1), t2: t })
                    }
                } else if t == SugaredTypeTerm::unit() {
                    Ok(SugaredTypeTerm::unit())
                } else {
                    Err(SugaredUnificationError { addr: unification_pair.addr, t1: SugaredTypeTerm::unit(), t2: t })
                }
            }


            /*
             Application
            */

            (SugaredTypeTerm::Spec(a1), SugaredTypeTerm::Spec(a2)) => {
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
                            SugaredUnificationPair {
                                lhs: x.clone(),
                                rhs: y.clone(),
                                addr: new_addr,
                            }
                        ) {
                            Ok(halo) => {
                                if halo == SugaredTypeTerm::unit() {
                                    let mut y = y.clone();
                                    y.apply_subst(&self.σ);
                                    y = y.strip();

                                    let top = y.get_interface_type();
                                    // eprintln!("add top {}", top.pretty(self.dict, 0));
                                    halo_args.push(top);
                                } else {
                                    //println!("add halo {}", halo.pretty(self.dict, 0));
                                    if n_halos_required > 0 {
                                        let x = &mut halo_args[n_halos_required-1];
                                        if let SugaredTypeTerm::Ladder(arg_rungs) = x {
                                            let mut a = a2[n_halos_required-1].clone();
                                            a.apply_subst(&self.σ);
                                            arg_rungs.push(a.get_interface_type());
                                        } else {
                                            *x = SugaredTypeTerm::Ladder(vec![
                                                x.clone(),
                                                a2[n_halos_required-1].get_interface_type()
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
                        Ok(SugaredTypeTerm::Spec(halo_args))
                    } else {
                        Ok(SugaredTypeTerm::unit())
                    }
                } else {
                    Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(SugaredUnificationError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }

    pub fn solve(mut self) -> Result<(Vec<SugaredTypeTerm>, HashMap<TypeID, SugaredTypeTerm>), SugaredUnificationError> {
        // solve equations
        while let Some( mut equal_pair ) = self.equal_pairs.pop() {
            equal_pair.lhs.apply_subst(&self.σ);
            equal_pair.rhs.apply_subst(&self.σ);

            self.eval_equation(equal_pair)?;
        }

        // solve subtypes
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs.apply_subst(&self.σ);
            subtype_pair.rhs.apply_subst(&self.σ);
            let _halo = self.eval_subtype( subtype_pair.clone() )?.strip();
        }

        // add variables from subtype bounds
        for (var_id, t) in self.upper_bounds.iter() {
            self.σ.insert(TypeID::Var(*var_id), t.clone().strip());
        }

        for (var_id, t) in self.lower_bounds.iter() {
            self.σ.insert(TypeID::Var(*var_id), t.clone().strip());
        }

        self.reapply_subst();

        let mut halo_types = Vec::new();
        for mut subtype_pair in self.subtype_pairs.clone().into_iter() {
            subtype_pair.lhs = subtype_pair.lhs.apply_subst(&self.σ).clone().strip();
            subtype_pair.rhs = subtype_pair.rhs.apply_subst(&self.σ).clone().strip();

            subtype_pair.lhs.apply_subst(&self.σ);
            subtype_pair.rhs.apply_subst(&self.σ);

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
    t1: &SugaredTypeTerm,
    t2: &SugaredTypeTerm
) -> Result<HashMap<TypeID, SugaredTypeTerm>, SugaredUnificationError> {
    let unification = SugaredUnificationProblem::new_eq(vec![ SugaredUnificationPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    Ok(unification.solve()?.1)
}

pub fn subtype_unify(
    t1: &SugaredTypeTerm,
    t2: &SugaredTypeTerm
) -> Result<(SugaredTypeTerm, HashMap<TypeID, SugaredTypeTerm>), SugaredUnificationError> {
    let unification = SugaredUnificationProblem::new_sub(vec![ SugaredUnificationPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(SugaredTypeTerm::unit()), σ) )
}

pub fn parallel_unify(
    t1: &SugaredTypeTerm,
    t2: &SugaredTypeTerm
) -> Result<(SugaredTypeTerm, HashMap<TypeID, SugaredTypeTerm>), SugaredUnificationError> {
    let unification = SugaredUnificationProblem::new_parallel(vec![ SugaredUnificationPair{ lhs: t1.clone(), rhs: t2.clone(), addr:vec![] } ]);
    unification.solve().map( |(halos,σ)| ( halos.first().cloned().unwrap_or(SugaredTypeTerm::unit()), σ) )
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
