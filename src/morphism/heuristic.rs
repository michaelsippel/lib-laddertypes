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

use std::ops::Deref;

use crate::{morphism::MorphismType, TypeTerm};

impl MorphismType {

    pub fn estimated_cost(&self) -> u64 {

        if let Ok((ψ,σ)) = crate::subtype_unify(&self.src_type, &self.dst_type) {
            0
        } else {
            match (self.src_type.clone().normalize(),
                self.dst_type.clone().normalize())
            {
                (TypeTerm::Ladder(r1),
                TypeTerm::Ladder(r2)) => {
                    let mut cost = 10;
                    for i in 0..usize::min( r1.len(), r2.len() ) {
                        cost += MorphismType {
                            Γ: Vec::new(),
                            bounds: Vec::new(),
                            src_type: r1[i].clone(),
                            dst_type: r2[i].clone()
                        }.estimated_cost();
                    }
                    cost
                }
                (TypeTerm::Spec(a1),
                TypeTerm::Spec(a2)) => {
                    let mut cost = 10;
                    for i in 0..usize::min( a1.len(), a2.len() ) {
                        cost += MorphismType {
                            Γ: Vec::new(),
                            bounds: Vec::new(),
                            src_type: a1[i].clone(),
                            dst_type: a2[i].clone()
                        }.estimated_cost();
                    }
                    cost
                }
                (TypeTerm::Seq{ seq_repr: sr1, item: item1 },
                TypeTerm::Seq{ seq_repr: sr2, item: item2 }) => {
                    let mut cost = 10;
        /* // todo : add cost seq-repr conversion?
                        estimated_morphism_cost(
                        &MorphismType { src_type: sr1, dst_type: sr2 }
                    );
        */
                    cost += MorphismType {
                        Γ: Vec::new(),
                        bounds: Vec::new(),
                        src_type: item1.deref().clone(),
                        dst_type: item2.deref().clone()
                    }.estimated_cost();

                    cost
                }

                (TypeTerm::Var(_), x)
                | (x, TypeTerm::Var(_))
                => {
                    return 1;
                }
                (a, b) => {
                    if a == b {
                        return 0;
                    } else {
                        return 10;
                    }
                }
            }
        }
    }
}
