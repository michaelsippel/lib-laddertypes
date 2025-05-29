use crate::{MorphismType, TypeTerm};

pub fn estimated_morphism_cost(ty: &MorphismType) -> u64 {

    if let Ok((ψ,σ)) = crate::subtype_unify(&ty.src_type, &ty.dst_type) {
        1
    } else {
        match (ty.src_type.clone().normalize(),
            ty.dst_type.clone().normalize())
        {
            (TypeTerm::Ladder(r1),
            TypeTerm::Ladder(r2)) => {
                let mut cost = 10;
                for i in 0..usize::min( r1.len(), r2.len() ) {
                    cost += estimated_morphism_cost(&MorphismType {
                        bounds: Vec::new(),
                        src_type: r1[i].clone(), dst_type: r2[i].clone() });
                }
                cost
            }
            (TypeTerm::Spec(a1),
            TypeTerm::Spec(a2)) => {
                let mut cost = 10;
                for i in 0..usize::min( a1.len(), a2.len() ) {
                    cost += estimated_morphism_cost(
                        &MorphismType {
                            bounds: Vec::new(),
                            src_type: a1[i].clone(), dst_type: a2[i].clone() });
                }
                cost
            }
            (TypeTerm::Seq{ seq_repr: sr1, items: items1 },
            TypeTerm::Seq{ seq_repr: sr2, items: items2 }) => {
                let mut cost = 10;
    /*
                    estimated_morphism_cost(
                    &MorphismType { src_type: sr1, dst_type: sr2 }
                );
    */
                for i in 0..usize::min( items1.len(), items2.len() ) {
                    cost += estimated_morphism_cost(
                        &MorphismType {
                            bounds: Vec::new(),
                            src_type: items1[i].clone(), dst_type: items2[i].clone() }
                    );
                }

                cost
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
