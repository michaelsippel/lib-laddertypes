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
    crate::{dict::*, morphism::*, parser::*, AddressingMode, ConstraintError, ConstraintSystem, Context, ContextEntry, ContextPtr, HashMapSubst, LayeredContext, TypeKind, TypeTerm, CP2
    },
    std::{collections::HashMap, sync::{Arc, RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq)]
pub struct DummyMorphism(MorphismType);
impl Morphism for DummyMorphism {
    fn get_type(&self) -> MorphismType {
        self.0.clone()
    }
}


fn morphism_test_setup() -> MorphismBase<DummyMorphism> {
    let mut root_ctx = Context::new();
    let mut base = MorphismBase::<DummyMorphism>::new(root_ctx.clone());

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("Radix", TypeKind::Value(root_ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx.parse("<Digit Radix> ~ Char").unwrap(),
            dst_type: ctx.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    });

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("Radix", TypeKind::Value(root_ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: ctx.parse("<Digit Radix> ~ Char").unwrap()
        })
    });

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("Radix", TypeKind::Value(ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx.parse("ℕ ~ <PosInt Radix BigEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: ctx.parse("ℕ ~ <PosInt Radix LittleEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("Radix", TypeKind::Value(ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType {
            Γ,bounds: Vec::new(),
            src_type: ctx.parse("ℕ ~ <PosInt Radix LittleEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: ctx.parse("ℕ ~ <PosInt Radix BigEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("SrcRadix", TypeKind::Value(ctx.clone().parse("ℕ").expect("parse")));
        ctx.add_variable("DstRadix", TypeKind::Value(ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ,bounds: Vec::new(),
            src_type: ctx.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: ctx.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ [<Digit DstRadix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        ctx.add_variable("SrcRadix", TypeKind::Value(ctx.clone().parse("ℕ").expect("parse")));
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ,bounds: Vec::new(),
            src_type: ctx.parse("ℤ_2^64 ~ ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: ctx.parse("ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    });
    base.add_morphism({
        let mut ctx = root_ctx.scope(AddressingMode::StackDown);
        let Γ = ctx.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ,bounds: Vec::new(),
            src_type: ctx.parse("ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: ctx.parse("ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ [<Digit 0>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base
}


#[test]
fn test_morphgraph_id() {
    let base = morphism_test_setup();
    let mut ctx = base.ctx();
    let morph_graph = MorphismGraph::new(base);


    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
            dst_type: ctx.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }),

        Ok(MorphismInstance::Id {
            τ: ctx.parse("ℤ_2^64 ~ machine.UInt64").expect("parse")
        })
    );
}

#[test]
fn test_morphgraph_prim() {
    let base = morphism_test_setup();
    let mut root_ctx = base.ctx();
    let morph_graph = MorphismGraph::new(base);

    let mut ctx_m1 = root_ctx.scope(AddressingMode::StackDown);
    ctx_m1.add_variable("Radix", TypeKind::Value(root_ctx.clone().parse("ℕ").expect("parse")));


    let mut ctxm = root_ctx.scope(AddressingMode::StackUp);
    let σs = ctxm.shift_variables(&ctx_m1.0.read().unwrap().Γ);
    eprintln!("test σs ={:?}", σs);
    assert!( ctxm.bind(ctxm.get_varid("Radix").unwrap(), TypeTerm::Num(10)).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: root_ctx.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: root_ctx.parse("<Digit 10> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }),

        Ok(MorphismInstance::Specialize {
            σ: vec![ ( ctxm.get_varid("Radix").unwrap(), TypeTerm::Num(10) ) ].into_iter().collect(),
            m: Box::new(
                MorphismInstance::Primitive {
                    σs,
                    m: DummyMorphism(MorphismType {
                        Γ: ctx_m1.clone().0.read().unwrap().Γ.clone(),
                        bounds: Vec::new(),
                        src_type: ctx_m1.parse("<Digit Radix> ~ Char").expect("parse"),
                        dst_type: ctx_m1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
                    })
                })
        })
    );
}

#[test]
fn test_morphgraph_chain() {
    let base = morphism_test_setup();
    let mut Γ = base.ctx();
    let morph_graph = MorphismGraph::new(base);

    let mut Γm1 = Γ.scope(AddressingMode::StackDown);
    Γm1.add_variable("Radix", TypeKind::Value(Γ.parse("ℕ").expect("")));
    let mut Γm2 = Γ.scope(AddressingMode::StackDown);

    let mut Γ2 = Γ.scope(AddressingMode::StackUp);

    // first instance
    let mut Γ3 = Γ.scope(AddressingMode::StackUp);
    let σs1 = Γ3.shift_variables(&Γm1.0.read().unwrap().Γ);
    let σs2 = Γ3.shift_variables(&Γm2.0.read().unwrap().Γ);

    // second instance
    let mut Γ4 = Γ3.scope(AddressingMode::StackUp);

    assert!( Γ3.bind(Γ3.get_varid("Radix").unwrap(), TypeTerm::Num(10)).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: Γ.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: Γ.parse("<Digit 10> ~ ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ [<Digit 0>~ℤ_2^64~machine.UInt64]").expect("parse"),
        }),

        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    ( Γ3.get_varid("Radix").unwrap(),  TypeTerm::Num(10) )
                ].into_iter().collect(),
                m: Box::new(MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Primitive {
                            σs: σs1,
                            m: DummyMorphism(MorphismType{
                                Γ: Γm1.clone().0.read().unwrap().Γ.clone(),
                                bounds: Vec::new(),
                                src_type: Γm1.parse("<Digit Radix> ~ Char").expect("parse"),
                                dst_type: Γm1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
                            })
                        },
                        MorphismInstance::Sub {
                            ψ: Γ3.parse("<Digit 10>").expect("parse"),
                            m: Box::new(MorphismInstance::Primitive {
                                σs: σs2,
                                m: DummyMorphism(MorphismType{
                                    Γ: Γm2.clone().0.read().unwrap().Γ.clone(),
                                    bounds: Vec::new(),
                                    src_type: Γm2.parse("ℤ_2^64 ~ machine.UInt64").unwrap(),
                                    dst_type: Γm2.parse("ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ [<Digit 0>~ℤ_2^64~machine.UInt64]").unwrap()
                                })
                            })
                        }
                    ]
                })
            }
        )
    );
}

#[test]
fn test_morphgraph_spec1() {
    let mut base = MorphismBase::<DummyMorphism>::new(Context::new());
    let mut ctx = base.ctx();

    let mut ctx_m1 = ctx.scope(AddressingMode::StackDown);
    base.add_morphism({
        ctx_m1.add_variable("X", TypeKind::Type);
        let Γ = ctx_m1.0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx_m1.parse("T ~ A").expect(""),
            dst_type: ctx_m1.parse("T ~ <B X> ~ U").expect("")
        })
    });

    let mut ctx_m2 = ctx.scope(AddressingMode::StackDown);
    base.add_morphism({
        ctx_m2.add_variable("Y", TypeKind::Type);
        let Γ = ctx_m2.0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx_m2.parse("T ~ <B Y> ~ U").expect(""),
            dst_type: ctx_m2.parse("T ~ <B Y> ~ V").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    let mut Γ1 = ctx.scope(AddressingMode::StackUp);
    let σs1 = Γ1.shift_variables(&ctx_m1.0.read().unwrap().Γ);
    assert!( Γ1.clone().bind(Γ1.get_varid("X").expect(""), Γ1.parse("test").expect("")).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("T ~ A").unwrap(),
            dst_type: ctx.parse("T ~ <B test> ~ U").unwrap(),
        }),
        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (Γ1.get_varid("X").unwrap(), Γ1.parse("test").expect(""))
                ].into_iter().collect(),
                m: Box::new(
                    MorphismInstance::Primitive {
                        σs:σs1.clone(),
                        m: DummyMorphism(
                            MorphismType {
                                Γ: ctx_m1.clone().0.read().unwrap().Γ.clone(),
                                bounds: Vec::new(),
                                src_type: ctx_m1.parse("T ~ A").expect("parse"),
                                dst_type: ctx_m1.parse("T ~ <B X> ~ U").expect("parse")
                            }
                        )
                    }
                )
            }
        )
    );
}

#[test]
fn test_morphgraph_spec2() {
    let mut base = MorphismBase::<DummyMorphism>::new(Context::new());
    let mut ctx = base.ctx();

    let mut ctx_m1 = ctx.scope(AddressingMode::StackDown);
    base.add_morphism({
        ctx_m1.add_variable("X", TypeKind::Type);
        let Γ = ctx_m1.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx_m1.parse("T ~ A").expect(""),
            dst_type: ctx_m1.parse("T ~ <B X> ~ U").expect("")
        })
    });

    let mut ctx_m2 = ctx.scope(AddressingMode::StackDown);
    base.add_morphism({
        ctx_m2.add_variable("Y", TypeKind::Type);
        let Γ = ctx_m2.clone().0.read().unwrap().Γ.clone();
        DummyMorphism(MorphismType{
            Γ, bounds: Vec::new(),
            src_type: ctx_m2.parse("T ~ <B Y> ~ U").expect(""),
            dst_type: ctx_m2.parse("T ~ <B Y> ~ V").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    let mut inst_ctx = ctx.scope(AddressingMode::StackUp);
    let σs1 = inst_ctx.shift_variables(&ctx_m1.0.read().unwrap().Γ);
    assert!( inst_ctx.clone().bind(inst_ctx.get_varid("X").expect(""), inst_ctx.parse("test").expect("")).is_ok() );

    let mut inst_ctx2 = inst_ctx.clone();//.scope(AddressingMode::StackUp);
    let σs2 = inst_ctx.shift_variables(&ctx_m2.0.read().unwrap().Γ);
    assert!( inst_ctx2.clone().bind(inst_ctx2.get_varid("Y").expect(""), inst_ctx2.parse("test").expect("")).is_ok() );

    let inst =
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("T ~ A").unwrap(),
            dst_type: ctx.parse("T ~ <B test> ~ V").unwrap(),
        });

    if let Ok(i) = inst.as_ref() {
        eprintln!("Found morphism instance: = \n==\n{}\n========", i.pretty(&ctx));
    }

    assert_eq!(
        inst,
        Ok(
            MorphismInstance::Specialize { σ: inst_ctx2.0.read().unwrap().σ.clone(),
                m: Box::new(MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Primitive {
                            σs:σs1,
                            m: DummyMorphism(MorphismType {
                                Γ: ctx_m1.clone().0.read().unwrap().Γ.clone(),
                                bounds: Vec::new(),
                                src_type: ctx_m1.parse("T ~ A").expect("parse"),
                                dst_type: ctx_m1.parse("T ~ <B X> ~ U").expect("parse")
                            })
                        },
                        MorphismInstance::Primitive {
                            σs:σs2,
                            m: DummyMorphism(MorphismType {
                                Γ: ctx_m2.clone().0.read().unwrap().Γ.clone(),
                                bounds: Vec::new(),
                                src_type: ctx_m2.parse("T ~ <B Y> ~ U").expect("parse"),
                                dst_type: ctx_m2.parse("T ~ <B Y> ~ V").expect("parse")
                            })
                        }
                    ]
                })
            }
        )
    );
}

#[test]
fn test_morphgraph_map_seq() {
    let mut ctx = Context::new();
    let mut base = MorphismBase::<DummyMorphism>::new(ctx.clone());

    base.add_morphism({
        DummyMorphism(MorphismType{
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("A ~ F").expect(""),
            dst_type: ctx.parse("A ~ E").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("[A ~ F]").unwrap(),
            dst_type: ctx.parse("[A ~ E]").unwrap(),
        }),
        Ok(
            MorphismInstance::MapSeq { seq_repr: None, item_morph: Box::new(
                MorphismInstance::Primitive {
                    σs: HashMapSubst::new(),
                    m: DummyMorphism(MorphismType {
                        Γ: Vec::new(),
                        bounds: Vec::new(),
                        src_type: ctx.parse("A ~ F").unwrap(),
                        dst_type: ctx.parse("A ~ E").unwrap()
                    })
                }
            ) }
        )
    );
}

#[test]
fn test_morphgraph_map_seq_repr() {
    let mut ctx = Context::new();
    let mut base = MorphismBase::<DummyMorphism>::new(ctx.clone());

    base.add_morphism({
        DummyMorphism(MorphismType{
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("A ~ F").expect(""),
            dst_type: ctx.parse("A ~ E").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("[~<array.Static 64> A ~ F]").unwrap(),
            dst_type: ctx.parse("[~<array.Static 64> A ~ E]").unwrap(),
        }),
        Ok(
            MorphismInstance::MapSeq { seq_repr: Some(Box::new(ctx.parse("<array.Static 64>").expect(""))), item_morph: Box::new(
                MorphismInstance::Primitive {
                    σs: HashMapSubst::new(),
                    m: DummyMorphism(MorphismType {
                        Γ: Vec::new(),
                        bounds: Vec::new(),
                        src_type: ctx.parse("A ~ F").unwrap(),
                        dst_type: ctx.parse("A ~ E").unwrap()
                    })
                }
            ) }
        )
    );
}

#[test]
fn test_morphism_path1() {
    let base = morphism_test_setup();
    let mut ctx = base.ctx();
    let morph_graph = MorphismGraph::new(base);

    let mut ctx_m1 = ctx.scope(AddressingMode::StackDown);
    ctx_m1.add_variable("Radix", TypeKind::Value(ctx.parse("ℕ").expect("")));

    let result = morph_graph.search(MorphismType {
        Γ: Vec::new(),
        bounds: Vec::new(),
        src_type: ctx.parse("ℕ ~ <PosInt 10 LittleEndian> ~ [<Digit 10> ~ Char]").unwrap(),
        dst_type: ctx.parse("ℕ ~ <PosInt 10 LittleEndian> ~ [<Digit 10> ~ ℤ_2^64 ~ machine.UInt64]").unwrap(),
    });

    match result.as_ref() {
        Ok(inst)=> {
            eprintln!("π = {}", inst.pretty(&ctx));
        }
        _ => {}
    }

    assert_eq!(
        result,
        Ok(
            MorphismInstance::Sub {
                ψ: ctx.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                m: Box::new(
                    MorphismInstance::MapSeq {
                        seq_repr: None,
                        item_morph: Box::new(
                            MorphismInstance::Specialize {
                                σ: vec![ (0, TypeTerm::Num(10)) ].into_iter().collect(),
                                m: Box::new(MorphismInstance::Primitive {
                                    σs: ctx.shift_variables(&ctx_m1.clone().0.read().unwrap().Γ),
                                    m: DummyMorphism(MorphismType {
                                        Γ: ctx_m1.clone().0.read().unwrap().Γ.clone(),
                                        bounds: Vec::new(),
                                        src_type: ctx_m1.parse("<Digit Radix> ~ Char").unwrap(),
                                        dst_type: ctx_m1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                                    }),
                                })
                            }
                        )
                    }
                )
            }));
}

#[test]
fn test_morphism_path2() {
    let base = morphism_test_setup();
    let mut ctx = base.ctx().scope(AddressingMode::StackUp);
    let morph_graph = MorphismGraph::new(base);

    let mut ctx_m1 = ctx.scope(AddressingMode::StackDown);
    ctx_m1.add_variable("Radix", TypeKind::Value(ctx_m1.clone().parse("ℕ").expect("parse")));
    let Γm1 = ctx_m1.clone().0.read().unwrap().Γ.clone();

    let c = ctx.scope(AddressingMode::StackDown);
    let mut ctx_m2 = ctx.scope(AddressingMode::StackDown);
    ctx_m2.add_variable("SrcRadix", TypeKind::Value(ctx_m2.clone().parse("ℕ").expect("parse")));
    ctx_m2.add_variable("DstRadix", TypeKind::Value(ctx_m2.clone().parse("ℕ").expect("parse")));
    let Γm2 = ctx_m2.clone().0.read().unwrap().Γ.clone();

    assert_eq!(
        morph_graph.search(MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: ctx.parse("ℕ ~ <PosInt 10 LittleEndian> ~ [<Digit 10> ~ Char]").unwrap(),
            dst_type: ctx.parse("ℕ ~ <PosInt 16 LittleEndian> ~ [<Digit 16> ~ ℤ_2^64 ~ machine.UInt64]").unwrap(),
        }),
        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (1, TypeTerm::Num(10)),
                    (2, TypeTerm::Num(16)),
                ].into_iter().collect(),
                m: Box::new(
                MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Sub {
                            ψ: ctx.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                            m: Box::new(
                                MorphismInstance::MapSeq {
                                    seq_repr: None,
                                    item_morph: Box::new(
                                        MorphismInstance::Specialize {
                                            σ: vec![
                                                (0, TypeTerm::Num(10)),
                                            ].into_iter().collect(),
                                            m: Box::new(MorphismInstance::Primitive {
                                                σs: ctx.shift_variables(&Γm1),
                                                m: DummyMorphism(MorphismType {
                                                    Γ: Γm1,
                                                    bounds: Vec::new(),
                                                    src_type: ctx_m1.parse("<Digit Radix> ~ Char").unwrap(),
                                                    dst_type: ctx_m1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                                                }),
                                            })
                                        }
                                    )
                                }
                            )
                        },
                        MorphismInstance::Primitive{
                            σs: ctx.shift_variables(&Γm2),
                            m: DummyMorphism(MorphismType {
                                Γ: Γm2,
                                bounds: Vec::new(),
                                src_type: ctx_m2.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64]").unwrap(),
                                dst_type: ctx_m2.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ [<Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64]").unwrap()
                            }),
                        }
                    ]
                })
            }
        ));
}

#[test]
fn test_morphism_path3() {
    let base = morphism_test_setup();
    let mut ctx = base.ctx();


    let mut ctx_m1 = ctx.scope(AddressingMode::StackDown);
    ctx_m1.add_variable("Radix", TypeKind::Value(ctx_m1.clone().parse("ℕ").expect("parse")));
    let Γm1 = ctx_m1.clone().0.read().unwrap().Γ.clone();

    let mut ctx_m2 = ctx.scope(AddressingMode::StackDown);
    ctx_m2.add_variable("SrcRadix", TypeKind::Value(ctx_m2.clone().parse("ℕ").expect("parse")));
    ctx_m2.add_variable("DstRadix", TypeKind::Value(ctx_m2.clone().parse("ℕ").expect("parse")));
    let Γm2 = ctx_m2.clone().0.read().unwrap().Γ.clone();

    let mut ctx_m3 = ctx.scope(AddressingMode::StackDown);
    ctx_m3.add_variable("Radix", TypeKind::Value(ctx_m3.clone().parse("ℕ").expect("parse")));
    let Γm3 = ctx_m3.clone().0.read().unwrap().Γ.clone();


    let morph_graph = MorphismGraph::new(base);
    let result = morph_graph.search(MorphismType {
        Γ: Vec::new(),
        bounds: Vec::new(),
        src_type: ctx.parse("ℕ ~ <PosInt 10 LittleEndian> ~ [<Digit 10> ~ Char]").unwrap(),
        dst_type: ctx.parse("ℕ ~ <PosInt 16 LittleEndian> ~ [<Digit 16> ~ Char]").unwrap()
    });

    assert_eq!(
        result,

        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (1, TypeTerm::Num(10)),
                    (2, TypeTerm::Num(16)),
                ].into_iter().collect(),

                m: Box::new(
                    MorphismInstance::Chain {
                        path: vec![

                MorphismInstance::Sub {
                    ψ: ctx.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                    m: Box::new(
                        MorphismInstance::MapSeq {
                            seq_repr: None,
                            item_morph: Box::new(
                                MorphismInstance::Specialize {
                                    σ: vec![ (0, TypeTerm::Num(10)) ].into_iter().collect(),
                                    m: Box::new(MorphismInstance::Primitive {
                                        σs: ctx.shift_variables(&Γm1),
                                        m: DummyMorphism(MorphismType {
                                            Γ: Γm1,
                                            bounds: Vec::new(),
                                            src_type: ctx_m1.parse("<Digit Radix> ~ Char").unwrap(),
                                            dst_type: ctx_m1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                                        }),
                                    })
                                }
                            )
                        }
                    )
                },
                MorphismInstance::Primitive{
                    σs: ctx.shift_variables(&Γm2),
                    m: DummyMorphism(MorphismType {
                        Γ: Γm2,
                        bounds: Vec::new(),
                        src_type: ctx_m2.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64]").unwrap(),
                        dst_type: ctx_m2.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ [<Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64]").unwrap()
                    }),
                },
                MorphismInstance::Sub {
                    ψ: ctx.parse("ℕ ~ <PosInt DstRadix LittleEndian>").expect(""),
                    m: Box::new(
                        MorphismInstance::MapSeq {
                            seq_repr: None,
                            item_morph:  Box::new(
                                        MorphismInstance::Specialize {
                                            σ: vec![
                                                (2, ctx.parse("16").expect("")),
                                                (3, ctx.parse("16").expect("")),
                                            ].into_iter().collect(),
                                            m: Box::new(MorphismInstance::Primitive {
                                                σs: ctx.shift_variables(&Γm3),
                                                    m: DummyMorphism(MorphismType {
                                                        Γ: Γm3,
                                                        bounds: Vec::new(),
                                                        src_type: ctx_m3.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
                                                        dst_type: ctx_m3.parse("<Digit Radix> ~ Char").unwrap()
                                                    }),
                                                })
                                        })
                                })
                    }
                ]
            })
        }
    ));
}



/*

#[test]
fn test_morphism_path_posint() {
    let (mut dict, base) = morphism_test_setup();

    let path = ShortestPathProblem::new(&base, MorphismType {
        bounds: Vec::new(),
        src_type: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().sugar(&mut dict),
        dst_type: dict.parse_desugared("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().sugar(&mut dict),
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(
            vec![
                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 10 BigEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (0, TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            bounds: Vec::new(),
                            src_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict)
                        }),
                    })
                },

                MorphismInstance::Primitive {
                    σ: vec![
                        (0, TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        bounds: Vec::new(),
                        src_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                },
                MorphismInstance::Primitive {
                    σ: vec![
                        (1, TypeTerm::Num(10)),
                        (2, TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        bounds: Vec::new(),
                        src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                    }),
                },
                MorphismInstance::Primitive {
                    σ: vec![
                        (2, TypeTerm::Num(16)),
                        (0, TypeTerm::Num(16)),
                    ].into_iter().collect(),
                    ψ: TypeTerm::unit(),
                    morph: DummyMorphism(MorphismType{
                        bounds: Vec::new(),
                        src_type: dict.parse_desugared("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                        dst_type: dict.parse_desugared("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                    }),
                },

                MorphismInstance::MapSeq {
                    ψ: dict.parse_desugared("ℕ ~ <PosInt 16 BigEndian>").expect("").sugar(&mut dict),
                    seq_repr: None,
                    item_morph: Box::new(MorphismInstance::Primitive {
                        σ: vec![
                            (0, TypeTerm::Num(16)),
                        ].into_iter().collect(),
                        ψ: TypeTerm::unit(),
                        morph: DummyMorphism(MorphismType {
                            bounds: Vec::new(),
                            src_type: dict.parse_desugared("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap().sugar(&mut dict),
                            dst_type: dict.parse_desugared("<Digit Radix> ~ Char").unwrap().sugar(&mut dict)
                        }),
                    })
                },
            ]
        )
    );
/*
    assert_eq!(
        base.find_morphism_path(MorphismType {
            src_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
        }),
        Some(
            vec![
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("Symbol ~ ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().normalize(),
            ]
        )
    );
    */
}
*/





/*
use std::collections::HashMap;

#[test]
fn test_morphism_path_listedit()
{
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new( vec![ dict.parse("List").expect("") ] );

    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("Char").unwrap(),
            dst_type: dict.parse("Char ~ EditTree").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char>").unwrap(),
            dst_type: dict.parse("<List Char>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List Char>").unwrap(),
            dst_type: dict.parse("<List Char~ReprTree>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List ReprTree>").unwrap(),
            dst_type: dict.parse("<List~Vec ReprTree>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
            dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
            dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
        })
    );


    let path = ShortestPathProblem::new(&base, MorphismType {
        src_type: dict.parse("<Seq~List~Vec <Digit 10>~Char>").unwrap(),
        dst_type: dict.parse("<Seq~List <Digit 10>~Char> ~ EditTree").unwrap(),
    }).solve();

    if let Some(path) = path.as_ref() {
        print_path(&mut dict, path);
    }

    assert_eq!(
        path,
        Some(vec![
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List~Vec Char>").unwrap(),
                    dst_type: dict.parse("<List Char>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List Char>").unwrap(),
                    dst_type: dict.parse("<List Char~ReprTree>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List ReprTree>").unwrap(),
                    dst_type: dict.parse("<List~Vec ReprTree>").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>~Char>").unwrap(),
                σ: HashMap::new()
            },
            MorphismInstance {
                m: DummyMorphism(MorphismType{
                    src_type: dict.parse("<List~Vec Char~ReprTree>").unwrap(),
                    dst_type: dict.parse("<List Char> ~ EditTree").unwrap()
                }),
                halo: dict.parse("<Seq~List <Digit 10>>").unwrap(),
                σ: HashMap::new()
            },
        ])
    );
}
*/
