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
                        cost += MorphismType { bounds: Vec::new(), src_type: r1[i].clone(), dst_type: r2[i].clone() }.estimated_cost();
                    }
                    cost
                }
                (TypeTerm::Spec(a1),
                TypeTerm::Spec(a2)) => {
                    let mut cost = 10;
                    for i in 0..usize::min( a1.len(), a2.len() ) {
                        cost += MorphismType { bounds: Vec::new(), src_type: a1[i].clone(), dst_type: a2[i].clone() }.estimated_cost();
                    }
                    cost
                }
                (TypeTerm::Seq{ seq_repr: sr1, items: items1 },
                TypeTerm::Seq{ seq_repr: sr2, items: items2 }) => {
                    let mut cost = 10;
        /* // todo : add cost seq-repr conversion?
                        estimated_morphism_cost(
                        &MorphismType { src_type: sr1, dst_type: sr2 }
                    );
        */
                    for i in 0..usize::min( items1.len(), items2.len() ) {
                        cost += MorphismType { bounds: Vec::new(), src_type: items1[i].clone(), dst_type: items2[i].clone() }.estimated_cost();
                    }

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
