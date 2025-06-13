use {
    crate::{
        TypeTerm
    },
    std::{sync::{Arc, RwLock}}
};

pub mod bimap;
pub mod dict;
pub mod substitution;

pub use {
    bimap::*,
    dict::*,
    substitution::*
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Type,
    Arrow( Box<TypeKind>, Box<TypeKind> ),
    ValueUInt
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextEntry {
    pub symbol: String,
    pub kind: TypeKind,
    pub value: Option<TypeTerm>
}

#[derive(Clone, Debug)]
pub struct Context {
    parent: Option<Arc<RwLock<Context>>>,
    names: Vec< String >,
    γ: Vec<ContextEntry>,
}

impl Context {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Context {
            parent: None,
            names: Vec::new(),
            γ: Vec::new()
        }))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeDict for Arc<RwLock<Context>> {
    fn add_typename(&mut self, tn: &str) -> u64 {
        let mut locked_self = self.write().unwrap();
        if let Some(parent) = locked_self.parent.as_mut() {
            parent.add_typename(tn) + locked_self.names.len() as u64
        } else {
            let idx = locked_self.names.len();
            locked_self.names.push(tn.into());
            idx as u64
        }
    }

    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        let locked_self = self.read().unwrap();

        for (i,n) in locked_self.γ.iter().enumerate() {
            if n.symbol == tn {
                return Some(TypeID::Var(i as u64));
            }
        }

        for (i,n) in locked_self.names.iter().enumerate() {
            if n == tn {
                return Some(TypeID::Fun(i as u64));
            }
        }

        if let Some(parent) = locked_self.parent.as_ref() {
            match parent.get_typeid(tn) {
                Some(TypeID::Fun(i)) => Some(TypeID::Fun(i + locked_self.names.len() as u64)),
                Some(TypeID::Var(i)) => Some(TypeID::Var(i + locked_self.γ.len() as u64)),
                None => None
            }
        } else {
            None
        }
    }

    fn get_typename(&self, tid: u64) -> Option<String> {
        let locked_self = self.read().unwrap();
        if (tid as usize) < locked_self.names.len() {
            Some(locked_self.names[tid as usize].clone())
        } else {
            if let Some(parent) = locked_self.parent.as_ref() {
                parent.get_typename(tid)
            } else {
                None
            }
        }
    }

    fn get_varname(&self, var_id: u64) -> Option<String> {
        let locked_self = self.read().unwrap();
        if (var_id as usize) < locked_self.γ.len() {
            Some(locked_self.γ[var_id as usize].symbol.clone())
        } else {
            if let Some(parent) = locked_self.parent.as_ref() {
                parent.get_typename(var_id)
            } else {
                None
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl Substitution for Arc<RwLock<Context>> {
    fn saturate(&mut self) {
        let mut locked_self = self.read().unwrap();
        for ContextEntry{ symbol, kind, value } in locked_self.γ.iter() {
            todo!()
        }
    }

    fn get(&self, var: u64) -> Result< crate::TypeTerm, SubstError > {
        let locked_self = self.read().unwrap();
        if (var as usize) < locked_self.γ.len() {
            if let Some(t) = locked_self.γ[var as usize].value.clone() {
                return Ok(t)
            }
        }
        if let Some(parent) = locked_self.parent.clone(){
            parent.get(var - locked_self.γ.len() as u64)
        } else {
            Err(SubstError::InvalidVariable)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait LayeredContext {
    fn add_variable(&self, symbol: &str, kind: TypeKind ) -> u64;
    fn bind(&self, var: u64, val: TypeTerm) -> Result<(), SubstError>;
    fn scope(&self) -> Self;

    fn shift_variables(&self, other: &Arc<RwLock<Context>>, t: TypeTerm) -> TypeTerm;
}

impl LayeredContext for Arc<RwLock<Context>> {
    fn shift_variables(&self, other: &Arc<RwLock<Context>>, mut t: TypeTerm) -> TypeTerm {
        let mut σ = HashMapSubst::new();
        let l = self.read().unwrap().γ.len() as u64;

        for (i,entry) in other.read().unwrap().γ.iter().enumerate() {
            let i = i as u64;

            // make variable name unique
            let mut s = entry.symbol.clone();
            while self.get_typeid(&s).is_some() {
                s.push_str("'");
            }

            self.add_variable(&s, entry.kind.clone());
            σ.insert(i, TypeTerm::Var(i + l));
        }

        t.apply_subst(&σ);
        t
    }

    fn add_variable(&self, symbol: &str, kind: TypeKind ) -> u64 {
        //self.write().unwrap().dict.add_varname(symbol.into());
        let mut locked_self = self.write().unwrap();

        let idx = locked_self.γ.len();
        locked_self.γ.push(ContextEntry{
            symbol: symbol.into(),
            kind,
            value: None,
        });
        idx as u64
    }

    fn bind(&self, var: u64, val: TypeTerm) -> Result<(), SubstError> {
        let mut locked_self = self.write().unwrap();
        let l = locked_self.γ.len() as u64;
        if var < l {
            if locked_self.γ[var as usize].value.is_none() {
                locked_self.γ[var as usize].value = Some(val);
                Ok(())
            } else {
                Err(SubstError::AlreadyAssigned)
            }
        } else {
            if let Some(parent) = locked_self.parent.as_mut() {
                parent.bind(var - l, val)
            } else {
                eprintln!("ERROR: assign invalid variable");
                Err(SubstError::InvalidVariable)
            }
        }
    }

    fn scope(&self) -> Arc<RwLock<Context>> {
        Arc::new(RwLock::new(Context{
            parent: Some(self.clone()),
            names: Vec::new(),
            γ: Vec::new()
        }))
    }
}
