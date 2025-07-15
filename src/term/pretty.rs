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

use {
    crate::{term::TypeTerm, ConstraintPair, ContextEntry, ContextPtr, EnumVariant, LayeredContext, StructMember, TypeDict, TypeKind, VariableConstraint},
    tiny_ansi::TinyAnsi
};


impl StructMember {
    pub fn pretty(&self, dict: &ContextPtr, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}
impl EnumVariant {
    pub fn pretty(&self, dict: &ContextPtr, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}

impl ConstraintPair {
    pub fn pretty(&self, dict: &ContextPtr, indent: u64) -> String {
        let mut s = String::new();
        match self {
            ConstraintPair::ValueOf(lhs, rhs) => {
                s.push('(');
                s.push_str(&lhs.pretty(dict, indent));
                s.push_str(": ");
                s.push_str(&rhs.pretty(dict, indent));
                s.push(')');
            },
            ConstraintPair::Subtype(lhs, rhs) => {
                s.push('(');
                s.push_str(&lhs.pretty(dict, indent));
                s.push_str(" <= ");
                s.push_str(&rhs.pretty(dict, indent));
                s.push(')');
            },
            ConstraintPair::Trait(lhs, rhs) => {
                s.push('(');
                s.push_str(&lhs.pretty(dict, indent));
                s.push_str(" >< ");
                s.push_str(&rhs.pretty(dict, indent));
                s.push(')');
            },
            ConstraintPair::Parallel(lhs, rhs) => {
                s.push('(');
                s.push_str(&lhs.pretty(dict, indent));
                s.push_str(" || ");
                s.push_str(&rhs.pretty(dict, indent));
                s.push(')');
            }
        }
        s
    }
}

impl TypeTerm {
    pub fn pretty(&self, dict: &ContextPtr, indent: u64) -> String {
        let indent_width = 4;
        match self {
            TypeTerm::Id(id) => {
                format!("{}", dict.get_typename(*id).unwrap_or("??".bright_red())).blue().bold()
            }
            TypeTerm::Var(id) => {
                format!("{}({})", dict.get_varname(*id).unwrap_or("??".bright_red()).bright_magenta(), id)
            },

            TypeTerm::Num(n) => {
                format!("{}", n).green().bold()
            }

            TypeTerm::Char(c) => {
                match c {
                    '\0' => format!("'\\0'"),
                    '\n' => format!("'\\n'"),
                    _ => format!("'{}'", c)
                }
            }

            TypeTerm::Univ{ Γ, bounds, τ } => {
                let ctx = dict.scope();
                ctx.0.write().unwrap().γ = Γ.to_vec();

                let mut s = String::new();

                for entry in Γ.iter() {
                    s.push_str(&"∀".yellow().bold());
                    match entry.kind {
                        TypeKind::Type =>{
                            s.push_str(&entry.symbol.bright_blue());
                        }
                        _ => {
                            s.push_str(&format!("{}:{:?}", entry.symbol.bright_blue(), entry.kind));
                        }
                    }
                }
                for bound in bounds.iter() {
                    s.push_str(&bound.pretty(&ctx, indent));
                }

                s.push_str(&τ.pretty(&ctx, indent));

                s
            }

            TypeTerm::Spec(args) => {
                let mut s = String::new();
                s.push_str(&"<".yellow());
                for i in 0..args.len() {
                    let arg = &args[i];
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str( &arg.pretty(dict,indent+1) );
                }
                s.push_str(&">".yellow());
                s
            }

            TypeTerm::Struct{ struct_repr, members } => {
                let mut s = String::new();
                s.push_str(&"{".yellow().bold());

                if let Some(struct_repr) = struct_repr {
                    s.push_str(&format!("{}{} ", "~".yellow(), struct_repr.pretty(dict, indent+1)));
                }

                for member in members {
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    s.push_str(&member.pretty(dict, indent + 1));
                    s.push_str(&";\n".bright_yellow());
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&"}".yellow().bold());
                s
            }

            TypeTerm::Enum{ enum_repr, variants } => {
                let mut s = String::new();
                s.push_str(&"(".yellow().bold());

                if let Some(enum_repr) = enum_repr {
                    s.push_str(&format!("{}{} ", "~".yellow(), enum_repr.pretty(dict, indent+1)));
                }


                for (i,variant) in variants.iter().enumerate() {
                    s.push('\n');
                    for x in 0..(indent+1)*indent_width {
                        s.push(' ');
                    }
                    if i > 0 {
                        s.push_str(&"| ".yellow().bold());
                    }
                    s.push_str(&variant.pretty(dict, indent + 1));
                }

                s.push('\n');
                for x in 0..indent*indent_width {
                    s.push(' ');
                }
                s.push_str(&")".yellow().bold());
                s
            }

            TypeTerm::Seq{ seq_repr, item } => {
                let mut s = String::new();
                s.push_str(&"[".yellow().bold());

                if let Some(seq_repr) = seq_repr {
                    s.push_str(&format!("{}{}", "~".yellow(), seq_repr.pretty(dict, indent+1)));
                }
                s.push(' ');
                s.push_str(&item.pretty(dict, indent+1));
                s.push_str(&" ]".yellow().bold());
                s
            }

            TypeTerm::Morph(src,dst) => {
                let mut s = String::new();
                s.push_str(&src.pretty(dict, indent));
                s.push_str(&"  ~~morph~~>  ".bright_yellow());
                s.push_str(&dst.pretty(dict, indent));
                s
            }

            TypeTerm::Func(args) => {
                let mut s = String::new();
                for i in 0..args.len() {
                    let arg = &args[i];
                    if i > 0{
                        s.push('\n');
                        for x in 0..(indent*indent_width) {
                            s.push(' ');
                        }
                        s.push_str(&"-->  ".bright_yellow());
                    } else {
//                        s.push_str("   ");
                    }
                    s.push_str(&arg.pretty(dict, indent));
                }
                s
            }

            TypeTerm::Ladder(rungs) => {
                let mut s = String::new();
                for i in 0..rungs.len() {
                    let rung = &rungs[i];
                    if i > 0{
                        s.push('\n');
                        for x in 0..(indent*indent_width) {
                            s.push(' ');
                        }
                        s.push_str(&"~ ".yellow());
                    }
                    s.push_str(&rung.pretty(dict, indent));
                }
                s
            }
        }
    }
}
