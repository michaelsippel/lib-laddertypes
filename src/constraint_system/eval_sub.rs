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
        term::TypeTerm, ConstraintError, CP2, ConstraintSystem, EnumVariant, StructMember
    }, std::ops::Deref
};

impl ConstraintSystem {
    pub fn add_lower_subtype_bound(&mut self, v: u64, new_lower_bound: TypeTerm) -> Result<(),()> {

        if new_lower_bound == TypeTerm::Var(v) {
            return Ok(());
        }

        if new_lower_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(lower_bound) = self.lower_bounds.get(&v).cloned() {
                        //eprintln!("var already exists. check max. type");
            if let Ok(halo) = self.eval_subtype(
                CP2 {
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
                CP2{
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
        if new_upper_bound == TypeTerm::Var(v) {
            return Ok(());
        }

        if new_upper_bound.contains_var(v) {
            // loop
            return Err(());
        }

        if let Some(upper_bound) = self.upper_bounds.get(&v).cloned() {
            if let Ok(_halo) = self.eval_subtype(
                CP2 {
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



    pub fn eval_subtype(&mut self, unification_pair: CP2) -> Result<
        // ok: halo type
        TypeTerm,
        // error
        ConstraintError
    > {
        match (unification_pair.lhs.clone().strip(), unification_pair.rhs.clone().strip()) {

            /*
             Variables
            */

            (TypeTerm::Var(v), t) => {
                //eprintln!("variable <= t");
                if self.add_upper_subtype_bound(v, t.clone()).is_ok() {
                    Ok(TypeTerm::unit())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: TypeTerm::Var(v), t2: t })
                }
            }


            (t, TypeTerm::Var(v)) => {
                //eprintln!("t <= variable");
                if self.add_lower_subtype_bound(v, t.clone()).is_ok() {
                    Ok(TypeTerm::unit())
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: TypeTerm::Var(v), t2: t })
                }
            }

            /*
             Atoms
            */
            (TypeTerm::Id(a1), TypeTerm::Id(a2)) => {
                if a1 == a2 { Ok(TypeTerm::unit()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs}) }
            }
            (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
                if n1 == n2 { Ok(TypeTerm::unit()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }
            (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
                if c1 == c2 { Ok(TypeTerm::unit()) } else { Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs }) }
            }

            /*
             Complex Types
            */

            (TypeTerm::Seq{ seq_repr: lhs_seq_repr, item: lhs_item },
                TypeTerm::Seq { seq_repr: rhs_seq_repr, item: rhs_item })
            => {
                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(0);

                let mut ψ_seq_repr = None;

                if let Some(rhs_seq_repr) = rhs_seq_repr.as_ref() {
                    //eprintln!("subtype unify: rhs has seq-repr: {:?}", rhs_seq_repr);
                    if let Some(lhs_seq_repr) = lhs_seq_repr.as_ref() {
                        //eprintln!("check if it maches lhs seq-repr: {:?}", lhs_seq_repr);
                        ψ_seq_repr = Some(Box::new(self.eval_subtype(CP2 { addr: new_addr.clone(), lhs: lhs_seq_repr.deref().clone(), rhs: rhs_seq_repr.deref().clone() })?));
                        //eprintln!("..yes!");
                    } else {
                        //eprintln!("...but lhs has none.");
                        return Err(ConstraintError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                let mut new_addr = unification_pair.addr.clone();
                new_addr.push(1);

                match self.eval_subtype( CP2 { addr: new_addr.clone(), lhs: lhs_item.deref().clone(), rhs: rhs_item.deref().clone() } ) {
                    Ok(ψ_item) => Ok(TypeTerm::Seq {
                        seq_repr: ψ_seq_repr,
                        item: Box::new(ψ_item)
                    }.strip()),
                    Err(e) => Err(ConstraintError{
                        addr: new_addr,
                        t1: e.t1,
                        t2: e.t2,
                    })
                }
            }
            (TypeTerm::Struct{ struct_repr: lhs_struct_repr, members: lhs_members },
                TypeTerm::Struct{ struct_repr: rhs_struct_repr, members: rhs_members })
            => {
                let new_addr = unification_pair.addr.clone();
                if let Some(rhs_struct_repr) = rhs_struct_repr.as_ref() {
                    if let Some(lhs_struct_repr) = lhs_struct_repr.as_ref() {
                        let _struct_repr_ψ = self.eval_subtype(CP2 { addr: new_addr.clone(), lhs: *lhs_struct_repr.clone(), rhs: *rhs_struct_repr.clone() })?;
                    } else {
                        return Err(ConstraintError{ addr: new_addr.clone(), t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_members.len() == rhs_members.len() {
                    let mut halo_members = Vec::new();
                    for (i,
                            (StructMember{ symbol: lhs_symbol, ty: lhs_ty},
                                StructMember{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_members.into_iter().zip(rhs_members.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);

                        let ψ = self.eval_subtype( CP2 { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } )?;
                        halo_members.push(StructMember { symbol: lhs_symbol, ty: ψ });
                    }
                    Ok(TypeTerm::Struct {
                        struct_repr: None,
                        members: halo_members
                    })
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
                        let _enum_repr_ψ = self.eval_subtype(CP2 { addr: new_addr.clone(), lhs: *lhs_enum_repr.clone(), rhs: *rhs_enum_repr.clone() })?;
                    } else {
                        return Err(ConstraintError{ addr: new_addr, t1: unification_pair.lhs, t2: unification_pair.rhs });
                    }
                }

                if lhs_variants.len() == rhs_variants.len() {
                    let mut halo_variants = Vec::new();
                    for (i,
                            (EnumVariant{ symbol: lhs_symbol, ty: lhs_ty },
                                EnumVariant{ symbol: rhs_symbol, ty: rhs_ty })
                        ) in
                            lhs_variants.into_iter().zip(rhs_variants.into_iter()).enumerate()
                    {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);
                        let ψ = self.eval_subtype( CP2 { addr: new_addr, lhs: lhs_ty.clone(), rhs: rhs_ty.clone() } )?;
                        halo_variants.push(EnumVariant { symbol: lhs_symbol, ty: ψ });
                    }
                    Ok(TypeTerm::Enum {
                        enum_repr: None,
                        variants: halo_variants
                    })
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
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
                            (t, TypeTerm::Var(v)) => {

                                if self.add_upper_subtype_bound(v,t.clone()).is_ok() {
                                    let mut new_upper_bound_ladder = vec![ t ];

                                    if let Some(next_rhs) = l2_iter.next() {

                                    } else {
                                        // ladder of rhs is empty
                                        // take everything

                                        while let Some((i,t)) = l1_iter.next() {
                                            new_upper_bound_ladder.push(t);
                                        }
                                    }

                                    new_upper_bound_ladder.reverse();
                                    if self.add_upper_subtype_bound(v, TypeTerm::Ladder(new_upper_bound_ladder)).is_ok() {
                                        // ok
                                    } else {
                                        return Err(ConstraintError {
                                            addr,
                                            t1: lhs,
                                            t2: rhs
                                        });
                                    }
                                } else {
                                    return Err(ConstraintError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                            (lhs, rhs) => {
                                if let Ok(ψ) = self.eval_subtype(
                                    CP2 {
                                        lhs: lhs.clone(),
                                        rhs: rhs.clone(),
                                        addr:addr.clone(),
                                    }
                                ) {
                                    // ok.
                                    //eprintln!("rungs are subtypes. continue");
                                    halo_ladder.push(ψ);
                                } else {
                                    return Err(ConstraintError {
                                        addr,
                                        t1: lhs,
                                        t2: rhs
                                    });
                                }
                            }
                        }
                    } else {
                        // not a subtype,
                        return Err(ConstraintError {
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
                Ok(TypeTerm::Ladder(halo_ladder).strip())//.param_normalize())
            },

            (TypeTerm::Seq { seq_repr, item: item_lhs }, TypeTerm::Spec(mut args)) => {
                let mut new_addr = unification_pair.addr.clone();

                if args.len() > 1 {
                    if let Some(seq_repr) = seq_repr {
                        let repr_rhs = args.remove(0);
                        let item_rhs = args.remove(0);

                        let mut reprψ = self.eval_subtype(CP2{
                            addr: new_addr.clone(),
                            lhs: seq_repr.as_ref().clone(),
                            rhs: repr_rhs.clone()
                        })?;

                        reprψ = TypeTerm::Ladder(vec![
                            reprψ,
                            repr_rhs.get_interface_type()
                        ]).normalize();

                        let mut new_addr = new_addr.clone();
                        new_addr.push(1);
                        let itemψ = self.eval_subtype(CP2 {
                            addr: new_addr,
                            lhs: item_lhs.deref().clone(),
                            rhs: item_rhs.clone()
                        })?;

                        let mut itemψ = TypeTerm::Ladder(vec![
                            itemψ.clone(),
                            item_rhs.get_interface_type()
                        ]).normalize();
                        itemψ.apply_subst(&self.σ);

                        Ok(
                            TypeTerm::Seq {
                                seq_repr: if reprψ.is_empty() { None }
                                          else { Some(Box::new(reprψ)) },
                                item: Box::new(itemψ)
                            }
                        )
                    } else {
                        Err(ConstraintError {
                            addr: new_addr,
                            t1: unification_pair.lhs,
                            t2: unification_pair.rhs
                        })
                    }
                } else {
                    Err(ConstraintError {
                        addr: unification_pair.addr,
                        t1: unification_pair.lhs,
                        t2: unification_pair.rhs
                    })
                }
            }

            (t, TypeTerm::Ladder(a1)) => {
                Err(ConstraintError{ addr: unification_pair.addr, t1: t, t2: TypeTerm::Ladder(a1) })
            }

            (TypeTerm::Ladder(mut a1), t) => {
                if a1.len() > 0 {
                    let mut new_addr = unification_pair.addr.clone();
                    new_addr.push( a1.len() - 1 );
                    if let Ok(halo) = self.eval_subtype(
                        CP2 {
                            lhs: a1.pop().unwrap(),
                            rhs: t.clone(),
                            addr: new_addr
                        }
                    ) {
                        a1.push(halo);
                        if a1.len() == 1 {
                            Ok(a1.pop().unwrap())
                        } else {
                            Ok(TypeTerm::Ladder(a1).normalize())
                        }
                    } else {
                        Err(ConstraintError{ addr: unification_pair.addr, t1: TypeTerm::Ladder(a1), t2: t })
                    }
                } else if t == TypeTerm::unit() {
                    Ok(TypeTerm::unit())
                } else {
                    Err(ConstraintError { addr: unification_pair.addr, t1: TypeTerm::unit(), t2: t })
                }
            }


            /*
             Application
            */

            (TypeTerm::Spec(a1), TypeTerm::Spec(a2)) => {
                if a1.len() == a2.len() {
                    let mut halo_args = Vec::new();
                    let mut n_halos_required = 0;
                    let mut next_arg_with_common_rung = 0;

                    for (i, (mut x, mut y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate() {
                        let mut new_addr = unification_pair.addr.clone();
                        new_addr.push(i);

                        x = x.strip();

//                        eprintln!("before strip: {:?}", y);
                        y = y.strip();
//                        eprintln!("after strip: {:?}", y);
//                        eprintln!("APP<> eval {:?} \n ?<=? {:?} ", x, y);

                        match self.eval_subtype(
                            CP2 {
                                lhs: x.clone(),
                                rhs: y.clone(),
                                addr: new_addr,
                            }
                        ) {
                            Ok(halo) => {
                                if halo.is_empty() {
                                    let mut y = y.clone();
                                    y.apply_subst(&self.σ);
                                    y = y.strip();

                                    let top = y.get_interface_type();
                                    halo_args.push(top);
                                } else {
                                    //println!("add halo {}", halo.pretty(self.dict, 0));
                                    while next_arg_with_common_rung < i {
                                        let x = &mut halo_args[next_arg_with_common_rung];
                                        *x = TypeTerm::Ladder(vec![
                                            x.clone(),
                                            a2[next_arg_with_common_rung].get_interface_type()
                                        ]).normalize();
                                        x.apply_subst(&self.σ);
                                        next_arg_with_common_rung += 1;
                                    }

                                    halo_args.push(halo);
                                    n_halos_required += 1;
                                }
                            },
                            Err(err) => { return Err(err); }
                        }
                    }

                    if n_halos_required > 0 {
                        Ok(TypeTerm::Spec(halo_args))
                    } else {
                        Ok(TypeTerm::unit())
                    }
                } else {
                    Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
                }
            }

            _ => Err(ConstraintError{ addr: unification_pair.addr, t1: unification_pair.lhs, t2: unification_pair.rhs })
        }
    }
}
