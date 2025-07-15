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

use std::{collections::HashMap, hash::Hash};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Debug)]
pub struct Bimap<V: Eq + Hash, Λ: Eq + Hash> {
    pub mλ: HashMap<V, Λ>,
    pub my: HashMap<Λ, V>,
}

impl<V: Eq + Hash + Clone, Λ: Eq + Hash + Clone> Bimap<V, Λ> {
    pub fn new() -> Self {
        Bimap {
            mλ: HashMap::new(),
            my: HashMap::new(),
        }
    }

    pub fn insert(&mut self, y: V, λ: Λ) {
        self.mλ.insert(y.clone(), λ.clone());
        self.my.insert(λ, y);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
