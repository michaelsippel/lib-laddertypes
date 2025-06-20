use {
    crate::{dict::*, morphism::*, parser::*, ConstraintError, ConstraintPair, ConstraintSystem, Context, ContextPtr, HashMapSubst, LayeredContext, TypeKind, TypeTerm
    },
    std::{collections::HashMap, sync::{Arc, RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq)]
pub struct DummyMorphism(ContextPtr, MorphismType);
impl Morphism for DummyMorphism {
    fn ctx(&self) -> ContextPtr {
        self.0.clone()
    }

    fn get_type(&self) -> MorphismType {
        self.1.clone()
    }
}


fn morphism_test_setup() -> MorphismBase<DummyMorphism> {
    let mut Γ = Context::new();
    let mut base = MorphismBase::<DummyMorphism>::new(Γ.clone());

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("Radix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("<Digit Radix> ~ Char").unwrap(),
            dst_type: Γ.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    });

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("Radix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: Γ.parse("<Digit Radix> ~ Char").unwrap()
        })
    });

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("Radix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("ℕ ~ <PosInt Radix BigEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: Γ.parse("ℕ ~ <PosInt Radix LittleEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("Radix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("ℕ ~ <PosInt Radix LittleEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: Γ.parse("ℕ ~ <PosInt Radix BigEndian> ~ [<Digit Radix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("SrcRadix", TypeKind::ValueUInt);
        Γ.add_variable("DstRadix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: Γ.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ [<Digit DstRadix>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base.add_morphism({
        let mut Γ = Γ.scope();
        Γ.add_variable("SrcRadix", TypeKind::ValueUInt);
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("ℤ_2^64 ~ ℕ ~ <PosInt SrcRadix LittleEndian> ~ [<Digit SrcRadix>~ℤ_2^64~machine.UInt64]").unwrap(),
            dst_type: Γ.parse("ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    });
    base.add_morphism({
        let mut Γ = Γ.scope();
        DummyMorphism(Γ.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γ.parse("ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: Γ.parse("ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ [<Digit 0>~ℤ_2^64~machine.UInt64]").unwrap()
        })
    });

    base
}


#[test]
fn test_morphgraph_id() {
    let base = morphism_test_setup();
    let mut Γ = base.ctx();
    let morph_graph = MorphismGraph::new(base);


    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: Γ.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
            dst_type: Γ.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }),

        Ok(MorphismInstance::Id {
            τ: Γ.parse("ℤ_2^64 ~ machine.UInt64").expect("parse")
        })
    );
}

#[test]
fn test_morphgraph_prim() {
    let base = morphism_test_setup();
    let mut Γ = base.ctx();
    let morph_graph = MorphismGraph::new(base);

    let mut Γm1 = Γ.scope();
    Γm1.add_variable("Radix", TypeKind::ValueUInt);

    let mut Γ1 = Γ.scope();
    let mut Γ3 = Γ1.scope();
    let σs = Γ3.shift_variables(&Γm1);
    assert!( Γ3.bind(Γ3.get_varid("Radix").unwrap(), TypeTerm::Num(10)).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: Γ.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: Γ.parse("<Digit 10> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }),

        Ok(MorphismInstance::Specialize {
            Γ: Γ3.clone(),
            m: Box::new(
                MorphismInstance::Primitive {
                    σs,
                    m: DummyMorphism(Γm1.clone(), MorphismType {
                        bounds: Vec::new(),
                        src_type: Γm1.parse("<Digit Radix> ~ Char").expect("parse"),
                        dst_type: Γm1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
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

    let mut Γm1 = Γ.scope();
    Γm1.add_variable("Radix", TypeKind::ValueUInt);
    let mut Γm2 = Γ.scope();

    let mut Γ2 = Γ.scope();

    // first instance
    let mut Γ3 = Γ2.scope();
    let σs1 = Γ.shift_variables(&Γm1);
    let σs2 = Γ.shift_variables(&Γm2);

    // second instance
    let mut Γ4 = Γ3.scope();

    assert!( Γ3.bind(Γ3.get_varid("Radix").unwrap(), TypeTerm::Num(10)).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: Γ.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: Γ.parse("<Digit 10> ~ ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ [<Digit 0>~ℤ_2^64~machine.UInt64]").expect("parse"),
        }),

        Ok(
            MorphismInstance::Specialize {
                Γ: Γ4.clone(),
                m: Box::new(MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Primitive {
                            σs: σs1,
                            m: DummyMorphism(Γm1.clone(), MorphismType{
                                bounds: Vec::new(),
                                src_type: Γm1.parse("<Digit Radix> ~ Char").expect("parse"),
                                dst_type: Γm1.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
                            })
                        },
                        MorphismInstance::Sub {
                            ψ: Γ3.parse("<Digit 10>").expect("parse"),
                            m: Box::new(MorphismInstance::Primitive {
                                σs: σs2,
                                m: DummyMorphism(Γm2.clone(), MorphismType{
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
    let mut Γ = base.ctx();

    let mut Γm1 = Γ.scope();
    base.add_morphism({
        Γm1.add_variable("X", TypeKind::Type);
        DummyMorphism(Γm1.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γm1.parse("T ~ A").expect(""),
            dst_type: Γm1.parse("T ~ <B X> ~ U").expect("")
        })
    });

    let mut Γm2 = Γ.scope();
    base.add_morphism({
        Γm2.add_variable("Y", TypeKind::Type);
        DummyMorphism(Γm2.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γm2.parse("T ~ <B Y> ~ U").expect(""),
            dst_type: Γm2.parse("T ~ <B Y> ~ V").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    let mut Γ1 = Γ.scope();
    let σs1 = Γ1.shift_variables(&Γm1);
    assert!( Γ1.clone().bind(Γ1.get_varid("X").expect(""), Γ1.parse("test").expect("")).is_ok() );

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: Γ.parse("T ~ A").unwrap(),
            dst_type: Γ.parse("T ~ <B test> ~ U").unwrap(),
        }),
        Ok(
            MorphismInstance::Specialize {
                Γ: Γ1.clone(),
                m: Box::new(
                    MorphismInstance::Primitive {
                        σs:σs1.clone(),
                        m: DummyMorphism(Γm1.clone(),
                            MorphismType {
                                bounds: Vec::new(),
                                src_type: Γm1.parse("T ~ A").expect("parse"),
                                dst_type: Γm1.parse("T ~ <B X> ~ U").expect("parse")
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
    let mut Γ = base.ctx();

    let mut Γm1 = Γ.scope();
    base.add_morphism({
        Γm1.add_variable("X", TypeKind::Type);
        DummyMorphism(Γm1.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γm1.parse("T ~ A").expect(""),
            dst_type: Γm1.parse("T ~ <B X> ~ U").expect("")
        })
    });

    let mut Γm2 = Γ.scope();
    base.add_morphism({
        Γm2.add_variable("Y", TypeKind::Type);
        DummyMorphism(Γm2.clone(), MorphismType{
            bounds: Vec::new(),
            src_type: Γm2.parse("T ~ <B Y> ~ U").expect(""),
            dst_type: Γm2.parse("T ~ <B Y> ~ V").expect("")
        })
    });

    let morph_graph = MorphismGraph::new(base);

    let mut Γ4 = Γ.scope();
    let σs1 = Γ4.shift_variables(&Γm1);

    //let mut Γ5 = Γ4.scope();
    let σs2 = Γ4.shift_variables(&Γm2);

    //assert!( Γ4.clone().bind(Γ4.get_varid("X").expect(""), Γ4.parse("test").expect("")).is_ok() );
    assert!( Γ4.clone().bind(Γ4.get_varid("Y").expect(""), Γ4.parse("test").expect("")).is_ok() );

    let inst =
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: Γ.parse("T ~ A").unwrap(),
            dst_type: Γ.parse("T ~ <B test> ~ V").unwrap(),
        });

    if let Ok(i) = inst.as_ref() {
        eprintln!("Found morphism instance: = \n==\n{}\n========", i.pretty(&Γ));
    }

    assert_eq!(
        inst,
        Ok(
            MorphismInstance::Specialize { Γ:Γ4,
                m: Box::new(MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Primitive {
                            σs:σs1,
                            m: DummyMorphism(Γm2.clone(), MorphismType {
                                bounds: Vec::new(),
                                src_type: Γm1.parse("T ~ A").expect("parse"),
                                dst_type: Γm1.parse("T ~ <B X> ~ U").expect("parse")
                            })
                        },
                        MorphismInstance::Primitive {
                            σs:σs2,
                            m: DummyMorphism(Γm2.clone(), MorphismType {
                                bounds: Vec::new(),
                                src_type: Γm2.parse("T ~ <B Y> ~ U").expect("parse"),
                                dst_type: Γm2.parse("T ~ <B Y> ~ V").expect("parse")
                            })
                        }
                    ]
                })
            }
        )
    );
}

/*
#[test]
fn test_morphgraph_map_seq() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    base.add_morphism(DummyMorphism(MorphismType{
        bounds: Vec::new(),
        src_type: dict.parse("A ~ F").expect(""),
        dst_type: dict.parse("A ~ E").expect("")
    }));

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("<Seq A ~ F>").unwrap(),
            dst_type: dict.parse("<Seq A ~ E>").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::MapSeq { seq_repr: None, item_morph: Box::new(
                MorphismInstance::Primitive {
                    m: DummyMorphism(MorphismType {
                        bounds: Vec::new(),
                        src_type: dict.parse("A ~ F").unwrap(),
                        dst_type: dict.parse("A ~ E").unwrap()
                    })
                }
            ) }
        )
    );
}

#[test]
fn test_morphgraph_map_seq_repr() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    base.add_morphism(DummyMorphism(MorphismType{
        bounds: Vec::new(),
        src_type: dict.parse("A ~ F").expect(""),
        dst_type: dict.parse("A ~ E").expect("")
    }));

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("<Seq~<StaticLength 64> A ~ F>").unwrap(),
            dst_type: dict.parse("<Seq~<StaticLength 64> A ~ E>").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::MapSeq {
                seq_repr: Some(Box::new(dict.parse("<StaticLength 64>").unwrap())),
                item_morph: Box::new(
                    MorphismInstance::Primitive {
                        m: DummyMorphism(MorphismType {
                            bounds: Vec::new(),
                            src_type: dict.parse("A ~ F").unwrap(),
                            dst_type: dict.parse("A ~ E").unwrap()
                        })
                    })
            })
    );
}

#[test]
fn test_morphism_path1() {
    let (mut dict, base) = morphism_test_setup();

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::Sub {
                ψ: dict.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                m: Box::new(
                    MorphismInstance::MapSeq {
                        seq_repr: None,
                        item_morph: Box::new(
                            MorphismInstance::Specialize {
                                σ: vec![
                                    (0, TypeTerm::Num(10)),
                                ].into_iter().collect(),
                                m: Box::new(MorphismInstance::Primitive {
                                    m: DummyMorphism(MorphismType {
                                        bounds: Vec::new(),
                                        src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
                                        dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
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
    let (mut dict, base) = morphism_test_setup();

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (2, TypeTerm::Num(16)),
                ].into_iter().collect(),
                m: Box::new(
                MorphismInstance::Chain {
                    path: vec![
                        MorphismInstance::Sub {
                            ψ: dict.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                            m: Box::new(
                                MorphismInstance::MapSeq {
                                    seq_repr: None,
                                    item_morph: Box::new(
                                        MorphismInstance::Specialize {
                                            σ: vec![
                                                (0, TypeTerm::Num(10)),
                                            ].into_iter().collect(),
                                            m: Box::new(MorphismInstance::Primitive {
                                                m: DummyMorphism(MorphismType {
                                                    bounds: Vec::new(),
                                                    src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
                                                    dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                                                }),
                                            })
                                        }
                                    )
                                }
                            )
                        },
                        MorphismInstance::Specialize {
                            σ: vec![
                                (1, TypeTerm::Num(10))
                            ].into_iter().collect(),
                            m: Box::new(
                                MorphismInstance::Primitive{
                                    m: DummyMorphism(MorphismType {
                                        bounds: Vec::new(),
                                        src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                                        dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                                    }),
                                }
                            )
                        }
                    ]
                })
            }
        ));
}

#[test]
fn test_morphism_path3() {
    let (mut dict, base) = morphism_test_setup();

    let morph_graph = MorphismGraph::new(base);

    let result = morph_graph.search(MorphismType {
        bounds: Vec::new(),
        src_type: dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
        dst_type: dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
    }, &mut dict);

    eprintln!("{:#?}", result);

    assert_eq!(
        result,

        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (2, TypeTerm::Num(16)),
                ].into_iter().collect(),

                m: Box::new(
                    MorphismInstance::Chain {
                        path: vec![

                MorphismInstance::Sub {
                    ψ: dict.parse("ℕ ~ <PosInt 10 LittleEndian>").expect(""),
                    m: Box::new(
                        MorphismInstance::MapSeq {
                            seq_repr: None,
                            item_morph: Box::new(
                                MorphismInstance::Specialize {
                                    σ: vec![
                                        (0, TypeTerm::Num(10)),
                                    ].into_iter().collect(),
                                    m: Box::new(MorphismInstance::Primitive {
                                        m: DummyMorphism(MorphismType {
                                            bounds: Vec::new(),
                                            src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
                                            dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
                                        }),
                                    })
                                }
                            )
                        }
                    )
                },
                MorphismInstance::Specialize {
                    σ: vec![
                        (1, TypeTerm::Num(10)),
                    ].into_iter().collect(),
                    m: Box::new(
                            MorphismInstance::Primitive{
                                m: DummyMorphism(MorphismType {
                                    bounds: Vec::new(),
                                    src_type: dict.parse_desugared("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict),
                                    dst_type: dict.parse_desugared("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().sugar(&mut dict)
                                }),
                            }
                        )
                },

                MorphismInstance::Sub {
                    ψ: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian>").expect(""),
                    m: Box::new(
                        MorphismInstance::MapSeq {
                            seq_repr: None,
                            item_morph:  Box::new(
                                        MorphismInstance::Specialize {
                                            σ: vec![
                                                (0, dict.parse("16").expect("")),
                                            ].into_iter().collect(),
                                            m: Box::new(MorphismInstance::Primitive {
                                                    m: DummyMorphism(MorphismType {
                                                        bounds: Vec::new(),
                                                        src_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
                                                        dst_type: dict.parse("<Digit Radix> ~ Char").unwrap()
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

*/






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


/*
    assert_eq!(
        base.find_morphism_with_subtyping(
            &MorphismType {
                src_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
                dst_type: dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
            }
        ),

        Some((
                DummyMorphism(MorphismType{
                    src_type: dict.parse("<Seq <Digit Radix> ~ Char>").unwrap(),
                    dst_type: dict.parse("<Seq <Digit Radix> ~ ℤ_2^64 ~ machine.UInt64>").unwrap()
                }),

                dict.parse("Symbol ~ ℕ ~ <PosInt 10 BigEndian>").unwrap(),

                vec![
                    (dict.get_typeid(&"Radix".into()).unwrap(),
                    dict.parse("10").unwrap())
                ].into_iter().collect::<std::collections::HashMap<TypeID, TypeTerm>>()
        ))
    );
    */
}

#[test]
fn morphism_test_seq_repr() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> native.UInt8>").unwrap().sugar(&mut dict),
            dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> native.UInt8>").unwrap().sugar(&mut dict)
        })
    );

    assert_eq!(
        base.get_morphism_instance(&MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> Char~Ascii~native.UInt8>").expect("parse").sugar(&mut dict),
            dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> Char~Ascii~native.UInt8>").expect("parse").sugar(&mut dict)
        }),
        Some(
            MorphismInstance::Primitive {
                ψ: dict.parse_desugared("<Seq Char~Ascii>").expect("").sugar(&mut dict),
                σ: HashMap::new(),
                morph: DummyMorphism(MorphismType{
                    bounds: Vec::new(),
                    src_type: dict.parse_desugared("<Seq~<ValueTerminated 0> native.UInt8>").unwrap().sugar(&mut dict),
                    dst_type: dict.parse_desugared("<Seq~<LengthPrefix native.UInt64> native.UInt8>").unwrap().sugar(&mut dict)
                })
            }
        )
    );
}

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
*/
