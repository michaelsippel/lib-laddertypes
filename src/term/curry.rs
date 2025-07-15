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

use crate::term::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

impl TypeTerm {
    /// transform term to have at max 2 entries in Application list
    pub fn curry(self) -> TypeTerm {
        match self {
            TypeTerm::Spec(args) => {
                if args.len() >= 2 {
                    let mut old_args = args.into_iter();
                    let mut new_args = vec![
                        old_args.next().unwrap(),
                        old_args.next().unwrap()
                    ];

                    for x in old_args {
                        new_args = vec![
                            TypeTerm::Spec(new_args),
                            x
                        ];
                    }

                    TypeTerm::Spec(new_args)
                } else {
                    TypeTerm::Spec(args)
                }
            }

            TypeTerm::Ladder(rungs) => {
                TypeTerm::Ladder(rungs.into_iter().map(|r| r.curry()).collect())
            }

            _ => self
        }
    }

    /// summarize all curried applications into one vec
    pub fn decurry(self) -> Self {
        match self {
            TypeTerm::Spec(mut args) => {
                if args.len() > 0 {
                    let a0 = args.remove(0).decurry();
                    match a0 {
                        TypeTerm::Spec(sub_args) => {
                            for (i,x) in sub_args.into_iter().enumerate() {
                                args.insert(i, x);
                            }
                        }
                        other => { args.insert(0, other); }
                    }
                }
                TypeTerm::Spec(args)
            }
            TypeTerm::Ladder(args) => {
                TypeTerm::Ladder(args.into_iter().map(|a| a.decurry()).collect())
            }
            _ => self
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
