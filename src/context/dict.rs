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

use crate::{bimap::Bimap, TypeKind};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum TypeID {
    Fun(u64),
    Var(u64)
}

pub trait TypeDict : Send + Sync {
    fn add_typename(&mut self, tn: &str) -> u64;
    fn get_typeid(&self, tn: &str) -> Option<TypeID>;
    fn get_typename(&self, tid: u64) -> Option<String>;
    fn get_varname(&self, var_id: u64) -> Option<String>;
    fn get_varkind(&self, var_id: u64) -> Option<TypeKind>;

    fn get_name(&self, id: TypeID) -> Option<String> {
        match id {
            TypeID::Fun(id) => self.get_typename(id),
            TypeID::Var(id) => self.get_varname(id)
        }
    }

    fn get_varid(&self, varname: &str) -> Option<u64> {
        match self.get_typeid(varname) {
            Some(TypeID::Var(id)) => Some(id),
            _ => None
        }
    }

    fn get_typeid_creat(&mut self, tn: &str) -> TypeID {
        if let Some(id) = self.get_typeid(tn) {
            id
        } else {
            TypeID::Fun(self.add_typename(tn))
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

    pub fn insert(&mut self, name: &str, id: TypeID) {
        self.typenames.insert(name.into(), id);
    }
    pub fn add_varname(&mut self, tn: &str) -> u64 {
        let tyid = self.type_var_counter;
        self.type_var_counter += 1;
        self.insert(tn.into(), TypeID::Var(tyid));
        tyid
    }
}

impl TypeDict for BimapTypeDict {
    fn add_typename(&mut self, tn: &str) -> u64 {
        let tyid = self.type_lit_counter;
        self.type_lit_counter += 1;
        self.insert(tn.into(), TypeID::Fun(tyid));
        tyid
    }

    fn get_typename(&self, id: u64) -> Option<String> {
        self.typenames.my.get(&TypeID::Fun(id)).cloned()
    }
    fn get_varname(&self, id: u64) -> Option<String> {
        self.typenames.my.get(&TypeID::Var(id)).cloned()
    }
    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.typenames.mλ.get(tn).cloned()
    }

    fn get_varkind(&self, id: u64) -> Option<TypeKind> {
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::{Arc,RwLock};

impl<T: TypeDict> TypeDict for Arc<RwLock<T>> {
    fn add_typename(&mut self, tn: &str) -> u64 {
        self.write().unwrap().add_typename(tn)
    }
    fn get_typename(&self, id: u64)-> Option<String> {
        self.read().unwrap().get_typename(id)
    }
    fn get_varname(&self, id: u64)-> Option<String> {
        self.read().unwrap().get_varname(id)
    }
    fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.read().unwrap().get_typeid(tn)
    }
    fn get_varkind(&self, id: u64) -> Option<TypeKind> {
        self.read().unwrap().get_varkind(id)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
