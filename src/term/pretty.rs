use {
    crate::{term::TypeTerm, EnumVariant, StructMember, TypeDict, VariableConstraint},
    tiny_ansi::TinyAnsi
};


impl StructMember {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}
impl EnumVariant {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        format!("{}: {}", self.symbol, self.ty.pretty(dict, indent+1))
    }
}

impl VariableConstraint {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        match self {
            VariableConstraint::UnconstrainedType => format!(""),
            VariableConstraint::Subtype(τ) => format!(":<= {}", τ.pretty(dict, indent)),
            VariableConstraint::Trait(τ) => format!(":>< {}", τ.pretty(dict, indent)),
            VariableConstraint::Parallel(τ) => format!(":|| {}", τ.pretty(dict, indent)),
            VariableConstraint::ValueUInt => format!(": ℤ"),
        }
    }
}

impl TypeTerm {
    pub fn pretty(&self, dict: &impl TypeDict, indent: u64) -> String {
        let indent_width = 4;
        match self {
            TypeTerm::Id(id) => {
                format!("{}", dict.get_typename(*id).unwrap_or("??".bright_red())).blue().bold()
            }
            TypeTerm::Var(id) => {
                format!("{}", dict.get_varname(*id).unwrap_or("??".bright_red())).bright_magenta()
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

            TypeTerm::Univ(bound, t) => {
                format!("{} {}{} . {}",
                    "∀".yellow().bold(),
                    dict.get_varname(0).unwrap_or("??".into()).bright_blue(),
                    bound.pretty(dict, indent),
                    t.pretty(dict,indent)
                )
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

            TypeTerm::Seq{ seq_repr, items } => {
                let mut s = String::new();
                s.push_str(&"[".yellow().bold());

                if let Some(seq_repr) = seq_repr {
                    s.push_str(&format!("{}{}", "~".yellow(), seq_repr.pretty(dict, indent+1)));
                }
                s.push(' ');

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        s.push(' ');
                    }
                    s.push_str(&item.pretty(dict, indent+1));
                }
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
