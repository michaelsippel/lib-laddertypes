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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Type,
    Arrow( Box<TypeKind>, Box<TypeKind> ),
    Value(TypeTerm)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContextEntry {
    pub symbol: String,
    pub kind: TypeKind,
}

#[derive(Clone, Debug)]
pub struct ContextPtr(pub Arc<RwLock<Context>>);

#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    ctxname: String,
    sub_count: u64,

    parent: Option<ContextPtr>,
    names: Vec< String >,
    pub γ: Vec<ContextEntry>,
    pub σ: HashMapSubst,
}

static count: RwLock<u64> = RwLock::new(0);

impl Context {
    pub fn new() -> ContextPtr {
        let mut c = count.write().unwrap();
        *c += 1;
        ContextPtr(Arc::new(RwLock::new(Context {
            ctxname: format!("Ctx {}", *c),
            sub_count: 0,

            parent: None,
            names: Vec::new(),
            γ: Vec::new(),
            σ: HashMapSubst::new(),
        })))
    }

    pub fn n_variables(&self) -> u64 {
        self.γ.len() as u64
        + if let Some(p) = self.parent.as_ref() {
            p.0.read().unwrap().n_variables()
        } else {
            0
        }
    }
}

impl ContextPtr {

    pub fn pretty(&self) -> String {
        let locked_self = self.0.read().unwrap();
        let mut s = String::new();

        s.push_str(&format!("({}) ∀{{", self.get_ctxname()));
        for entry in locked_self.γ.iter() {
            s.push_str(&format!("{} ↦ {:?};", entry.symbol, entry.kind));
        }
        s.push_str("}");

        s.push_str(",σ={");
        for (v,t) in locked_self.σ.iter() {
            s.push_str(&format!("{}({}) ↦ {};",
                self.get_varname(*v).unwrap_or("??".into()),
                v,
                t.pretty(&mut self.clone(), 0)));
        }
        s.push_str("}");

        if let Some(p) = locked_self.parent.as_ref() {
            s.push_str(".");
            s.push_str(&p.pretty());
        }

        s
    }
}

impl PartialEq for ContextPtr {
    fn eq(&self, other: &Self) -> bool {
        let locked_lhs = self.0.read().unwrap();
        let locked_rhs = other.0.read().unwrap();

        match (locked_lhs.clone(),locked_rhs.clone()) {
            (Context { ctxname:_, sub_count: _, parent:p1, names:n1, σ:σ1, γ:γ1 },
                Context { ctxname:_, sub_count: _, parent:p2, names:n2, σ:σ2, γ:γ2 }) => {

                    if let (Some(p1),Some(p2)) = (&p1,&p2) {
                        if p1 == p2 && σ1==σ2 {

                        } else {
                            return false;
                        }
                    } else if  p1.is_none() && p2.is_none() {

                    } else {
                        return false;
                    }

                    ( n1 == n2 ) && (γ1 == γ2)
            }
        }
    }
}

impl ContextPtr {
    pub fn get_ctxname(&self) -> String {
        self.0.read().unwrap().ctxname.clone()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeDict for ContextPtr {
    fn add_typename(&mut self, tn: &str) -> u64 {
        let mut locked_self = self.0.write().unwrap();
        if let Some(parent) = locked_self.parent.as_mut() {
            parent.add_typename(tn) + locked_self.names.len() as u64
        } else {
            let idx = locked_self.names.len();
            locked_self.names.push(tn.into());
            idx as u64
        }
    }

    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        let locked_self = self.0.read().unwrap();

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
        let locked_self = self.0.read().unwrap();
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
        let locked_self = self.0.read().unwrap();
        let l = locked_self.γ.len() as u64;
        if var_id < l {
            Some(locked_self.γ[var_id as usize].symbol.clone())
        } else {
            if let Some(parent) = locked_self.parent.as_ref() {
                parent.get_varname(var_id - l)
            } else {
                None
            }
        }
    }

    fn get_varkind(&self, var: u64) -> Option<TypeKind> {
        let mut locked_self = self.0.read().unwrap();
        if var < locked_self.γ.len() as u64 {
            Some(locked_self.γ[var as usize].kind.clone())
        } else {
            None
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl Substitution for ContextPtr {
    fn saturate(&mut self) {
        let mut locked_self = self.0.read().unwrap();
        for ContextEntry{ symbol, kind } in locked_self.γ.iter() {
            todo!()
        }
    }

    fn get(&self, var: u64) -> Result< crate::TypeTerm, SubstError > {
        let locked_self = self.0.read().unwrap();
        let l = locked_self.γ.len() as u64;
        if let Some(t) = locked_self.σ.get(&var) {
            return Ok(t.clone())
        } else {
            if var >= l {
                if let Some(parent) = locked_self.parent.clone() {
                    parent.get(var - l)
                } else {
                    Err(SubstError::InvalidVariable)
                }
            } else {
                Err(SubstError::UnassignedVariable)
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

pub trait LayeredContext : TypeDict {
    fn add_variable(&self, symbol: &str, kind: TypeKind ) -> u64;
    fn bind(&self, var: u64, val: TypeTerm) -> Result<(), SubstError>;
    fn scope(&self) -> Self;

    fn shift_variables(&self, other: &ContextPtr) -> HashMapSubst;
    fn shift_from_parent(&self) -> HashMapSubst;
}

impl LayeredContext for ContextPtr {
    /*
     * take all variables declared in context `other`
     * and copy them over to self.
     * Return a substitution to map terms under context `other`
     * to context `self`.
     */
    fn shift_variables(&self, other: &ContextPtr) -> HashMapSubst {
        eprintln!("shift Vars from {:?} to {:?}", other.get_ctxname(), self.get_ctxname());

        // substitution mapping Variables of `other` to variables of `self`
        let mut σs = HashMapSubst::new();

        // number of local variables in `self`
        let l = self.0.read().unwrap().γ.len() as u64;

        for (i, entry) in other.0.read().unwrap().γ.iter().enumerate() {
            let i = i as u64;

            // make variable name unique
            let mut s = entry.symbol.clone();
            while let Some(id) = self.get_typeid(&s) {
                eprintln!("already have {} -> {:?}", s, id);
                s.push_str("'");
            }

            eprintln!("add {} ({} -> {})", s, i, i+l);

            self.add_variable(&s, entry.kind.clone());
            σs.insert(i, TypeTerm::Var(i + l));
        }
        σs
    }

    fn shift_from_parent(&self) -> HashMapSubst {
        let mut σss = HashMapSubst::new();
        let locked_self = self.0.read().unwrap();
        let l = locked_self.γ.len() as u64;

        if let Some(p) = locked_self.parent.as_ref() {
            for i in 0..p.0.read().unwrap().n_variables() {
                σss.insert( i as u64, TypeTerm::Var(l+i as u64) );
            }
        }

        σss
    }

    fn add_variable(&self, symbol: &str, kind: TypeKind ) -> u64 {
        //self.write().unwrap().dict.add_varname(symbol.into());
        let mut locked_self = self.0.write().unwrap();

        let idx = locked_self.γ.len();
        eprintln!("Ctx {}, add {} : {:?} = {}", locked_self.ctxname, symbol, kind, idx);
        locked_self.γ.push(ContextEntry{
            symbol: symbol.into(),
            kind
        });
        idx as u64
    }

    fn bind(&self, var: u64, val: TypeTerm) -> Result<(), SubstError> {
        let mut locked_self = self.0.write().unwrap();
        let l = locked_self.γ.len() as u64;
        locked_self.σ.insert(var, val.clone());
/*
        if var >= l {
            if let Some(parent) = locked_self.parent.as_ref() {
                parent.bind(var - l, val)?;
            }
        }
*/
        // todo check if var is in valid  range
        Ok(())
    }

    fn scope(&self) -> ContextPtr {
        let mut locked_self = self.0.write().unwrap();
        locked_self.sub_count += 1;
        ContextPtr(Arc::new(RwLock::new(Context{
            ctxname: format!("{}+{}", locked_self.ctxname, locked_self.sub_count),
            sub_count: 0,
            parent: Some(self.clone()),
            names: Vec::new(),
            γ: Vec::new(),
            σ: HashMapSubst::new()
        })))
    }
}
