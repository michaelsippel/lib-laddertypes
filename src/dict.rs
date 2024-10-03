use crate::bimap::Bimap;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TypeID {
    Fun(u64),
    Var(u64)
}

pub trait TypeDict {
    fn insert(&mut self, name: String, id: TypeID);
    fn add_varname(&mut self, vn: String) -> TypeID;
    fn add_typename(&mut self, tn: String) -> TypeID;
    fn get_typeid(&self, tn: &String) -> Option<TypeID>;
    fn get_typename(&self, tid: &TypeID) -> Option<String>;

    fn get_varname(&self, var_id: u64) -> Option<String> {
        self.get_typename(&TypeID::Var(var_id))
    }

    fn add_synonym(&mut self, new: String, old: String) {
        if let Some(tyid) = self.get_typeid(&old) {
            self.insert(new, tyid);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Debug)]
pub struct BimapTypeDict {
    typenames: Bimap<String, TypeID>,
    type_lit_counter: u64,
    type_var_counter: u64,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl BimapTypeDict {
    pub fn new() -> Self {
        BimapTypeDict {
            typenames: Bimap::new(),
            type_lit_counter: 0,
            type_var_counter: 0,
        }
    }
}

impl TypeDict for BimapTypeDict {
    fn insert(&mut self, name: String, id: TypeID) {
        self.typenames.insert(name, id);
    }

    fn add_varname(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Var(self.type_var_counter);
        self.type_var_counter += 1;
        self.insert(tn, tyid.clone());
        tyid
    }

    fn add_typename(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Fun(self.type_lit_counter);
        self.type_lit_counter += 1;
        self.insert(tn, tyid.clone());
        tyid
    }

    fn get_typename(&self, tid: &TypeID) -> Option<String> {
        self.typenames.my.get(tid).cloned()
    }

    fn get_typeid(&self, tn: &String) -> Option<TypeID> {
        self.typenames.mλ.get(tn).cloned()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::Arc;
use std::ops::{Deref, DerefMut};
use std::sync::RwLock;

impl<T: TypeDict> TypeDict for Arc<RwLock<T>> {
    fn insert(&mut self, name: String, id: TypeID) {
        self.write().unwrap().insert(name, id);
    }
    fn add_varname(&mut self, vn: String) -> TypeID {
        self.write().unwrap().add_varname(vn)
    }
    fn add_typename(&mut self, tn: String) -> TypeID {
        self.write().unwrap().add_typename(tn)
    }
    fn get_typename(&self, tid: &TypeID)-> Option<String> {
        self.read().unwrap().get_typename(tid)
    }
    fn get_typeid(&self, tn: &String) -> Option<TypeID> {
        self.read().unwrap().get_typeid(tn)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
