use {
    std::collections::HashMap,
    crate::{term::*, dict::*}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UnificationError {
    pub addr: Vec<usize>,
    pub t1: TypeTerm,
    pub t2: TypeTerm
}

impl UnificationError {
    pub fn new(t1: &TypeTerm, t2: &TypeTerm) -> Self {
        UnificationError {
            addr: vec![],
            t1: t1.clone(),
            t2: t2.clone()
        }
    }
}
/*
struct UnificationProblem {
    eqs: Vec<(TypeTerm, TypeTerm)>,
    σ: HashMap<TypeID, TypeTerm>
}

impl UnificationProblem {
    pub fn new() -> Self {
        UnificationProblem {
            eqs: Vec::new(),
            σ: HashMap::new()
        }
    }

    pub fn eval_equation(&mut self, lhs: &TypeTerm, rhs: &TypeTerm) -> Option<UnificationError> {
        match (lhs, rhs) {
            
        }
    }

    pub fn solve(self) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
        
    }
}
*/
pub fn unify(
    t1: &TypeTerm,
    t2: &TypeTerm
) -> Result<HashMap<TypeID, TypeTerm>, UnificationError> {
    let mut σ = HashMap::new();
    
    match (t1, t2) {
        (TypeTerm::TypeID(TypeID::Var(varid)), t) |
        (t, TypeTerm::TypeID(TypeID::Var(varid))) => {
            σ.insert(TypeID::Var(*varid), t.clone());
            Ok(σ)
        }

        (TypeTerm::TypeID(a1), TypeTerm::TypeID(a2)) => {
            if a1 == a2 { Ok(σ) } else { Err(UnificationError::new(&t1, &t2)) }
        }
        (TypeTerm::Num(n1), TypeTerm::Num(n2)) => {
            if n1 == n2 { Ok(σ) } else { Err(UnificationError::new(&t1, &t2)) }
        }
        (TypeTerm::Char(c1), TypeTerm::Char(c2)) => {
            if c1 == c2 { Ok(σ) } else { Err(UnificationError::new(&t1, &t2)) }
        }

        (TypeTerm::Ladder(a1), TypeTerm::Ladder(a2)) |
        (TypeTerm::App(a1), TypeTerm::App(a2)) => {
            if a1.len() == a2.len() {
                for (i, (x, y)) in a1.iter().cloned().zip(a2.iter().cloned()).enumerate() {
                    let (mut x, mut y) = (x.clone(), y.clone());
                    x.apply_substitution(&|v| σ.get(v).cloned());
                    y.apply_substitution(&|v| σ.get(v).cloned());

                    match unify(&x, &y) {
                        Ok(τ) => {
                            for (v,t) in τ {
                                σ.insert(v,t);
                            }
                        }
                        Err(mut err) => {
                            err.addr.insert(0, i);
                            return Err(err);
                        }
                    }
                }
                Ok(σ)
            } else {
                Err(UnificationError::new(&t1, &t2))
            }
        }

        (TypeTerm::Ladder(l1), TypeTerm::Ladder(l2)) => {
            Err(UnificationError::new(&t1, &t2))
        }

        _ => Err(UnificationError::new(t1, t2))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

