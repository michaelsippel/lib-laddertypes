use {
    crate::{dict::*, morphism::*, parser::*, ConstraintError, ConstraintPair, ConstraintSystem, Context, HashMapSubst, LayeredContext, TypeKind, TypeTerm
    },
    std::{collections::HashMap, sync::{Arc, RwLock}}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[derive(Clone, Debug, PartialEq, Eq)]
struct DummyMorphism(MorphismType);
impl Morphism for DummyMorphism {
    fn ctx(&self) -> Arc<RwLock<Context>> {
        Context::new()
    }

    fn get_type(&self) -> MorphismType {
        self.0.clone()
    }
}

fn morphism_test_setup() -> ( BimapTypeDict, MorphismBase<DummyMorphism> ) {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    dict.add_varname("Radix".into());
    dict.add_varname("SrcRadix".into());
    dict.add_varname("DstRadix".into());

    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ Char").unwrap()
        })
    );

    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );

    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );

    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("ℤ_2^64 ~ ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℤ_2^64 ~ machine.UInt64").unwrap()
        })
    );
    base.add_morphism(
        DummyMorphism(MorphismType{
            bounds: Vec::new(),
            src_type: dict.parse("ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: dict.parse("ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ <Seq <Digit 0>~ℤ_2^64~machine.UInt64>").unwrap()
        })
    );
    (dict, base)
}


#[test]
fn test_morphism_compat() {
    let ctx = Context::new();

    let mut c1 = ctx.scope();
    c1.add_variable("T1", TypeKind::Type);
    c1.add_variable("T2", TypeKind::Type);
    let t1 = MorphismType {
        bounds: Vec::new(),
        src_type: c1.parse("<Seq T1>~<A T1 T2>").unwrap(),
        dst_type: c1.parse("<Seq T1>~<B T2 T2>").unwrap()
    };

    let mut c2 = ctx.scope();
    c2.add_variable("S1", TypeKind::Type);
    c2.add_variable("T1", TypeKind::Type); //< this variable name is scoped thus a *different* variable than T1 from t1
    let t2 = MorphismType {
        bounds: Vec::new(),
        src_type: c2.parse("<Seq NotT>~<B S1 T1>").unwrap(),
        dst_type: c2.parse("<Seq NotT>~<C T1>").unwrap()
    };

    // pull t1 & t2 into root ctx
    let t1 = ctx.shift_variables(&c1, t1.dst_type.clone());
    let t2 = ctx.shift_variables(&c2, t2.src_type.clone());

    let csp = ConstraintSystem::new_sub(vec![
        ConstraintPair {
            lhs: t1.clone(),
            rhs: t2.clone(),
            addr: vec![]
        }
    ]);

    eprintln!("t1 = {:?} = {}", t1, t1.pretty(&mut ctx.clone(), 0));
    eprintln!("t2 = {:?} = {}", t2, t2.pretty(&mut ctx.clone(), 0));

    match csp.solve() {
        Ok((Ψ,σ)) => {
            eprintln!("σ = {:?}", σ);
            assert!(true);
        }
        Err(err) => {
            assert!(false);
        }
    }
}

#[test]
fn test_morphgraph_id() {
    let (mut dict, mut base) = morphism_test_setup();
    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
            dst_type: dict.parse("ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }, &mut dict),

        Ok(MorphismInstance::Id {
            τ: dict.parse("ℤ_2^64 ~ machine.UInt64").expect("parse")
        })
    );
}

#[test]
fn test_morphgraph_prim() {
    let (mut dict, base) = morphism_test_setup();
    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: dict.parse("<Digit 10> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
        }, &mut dict),

        Ok(
            MorphismInstance::Specialize { σ: vec![
                (0, TypeTerm::Num(10))
            ].into_iter().collect(), m: Box::new(
                    MorphismInstance::Primitive {
                    m: DummyMorphism(MorphismType{
                        bounds: Vec::new(),
                        src_type: dict.parse("<Digit Radix> ~ Char").expect("parse"),
                        dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
                    })
                })
            }
        )
    );
}


#[test]
fn test_morphgraph_chain() {
    let (mut dict, base) = morphism_test_setup();
    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("<Digit 10> ~ Char").expect("parse"),
            dst_type: dict.parse("<Digit 10> ~ ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ <Seq <Digit 0>~ℤ_2^64~machine.UInt64>").expect("parse"),
        }, &mut dict),

        Ok(
            MorphismInstance::Chain {
                path: vec![
                    MorphismInstance::Specialize {
                        σ: vec![
                            (0, TypeTerm::Num(10)),
                        ].into_iter().collect(),
                        m: Box::new(
                            MorphismInstance::Primitive {
                                m: DummyMorphism(MorphismType{
                                    bounds: Vec::new(),
                                    src_type: dict.parse("<Digit Radix> ~ Char").expect("parse"),
                                    dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").expect("parse"),
                                })
                            })
                    },
                    MorphismInstance::Sub {
                        ψ: dict.parse("<Digit 10>").expect("parse"),
                        m: Box::new(
                            MorphismInstance::Primitive { m: DummyMorphism(MorphismType{
                                bounds: Vec::new(),
                                src_type: dict.parse("ℤ_2^64 ~ machine.UInt64").unwrap(),
                                dst_type: dict.parse("ℤ_2^64 ~ ℕ ~ <PosInt 0 LittleEndian> ~ <Seq <Digit 0>~ℤ_2^64~machine.UInt64>").unwrap()
                            }) }
                        )
                    }
                ]
            }
        )
    );
}



#[test]
fn test_morphgraph_spec() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<DummyMorphism>::new();

    dict.add_varname("X".into());
    dict.add_varname("Y".into());

    base.add_morphism(DummyMorphism(MorphismType{
        bounds: Vec::new(),
        src_type: dict.parse("T ~ A").expect(""),
        dst_type: dict.parse("T ~ <B X> ~ U").expect("")
    }));
    base.add_morphism(DummyMorphism(MorphismType{
        bounds: Vec::new(),
        src_type: dict.parse("T ~ <B Y> ~ U").expect(""),
        dst_type: dict.parse("T ~ <B Y> ~ V").expect("")
    }));

    let morph_graph = MorphismGraph::new(base);

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("T ~ A").unwrap(),
            dst_type: dict.parse("T ~ <B test> ~ U").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    (0, dict.parse("test").expect("parse") )
                ].into_iter().collect(),
                m: Box::new(MorphismInstance::Primitive { m: DummyMorphism(MorphismType {
                    bounds: Vec::new(),
                    src_type: dict.parse("T ~ A").expect("parse"),
                    dst_type: dict.parse("T ~ <B X> ~ U").expect("parse")
                }) })
            }
        )
    );

    assert_eq!(
        morph_graph.search(MorphismType {
            bounds: Vec::new(),
            src_type: dict.parse("T ~ A").unwrap(),
            dst_type: dict.parse("T ~ <B test> ~ V").unwrap(),
        }, &mut dict),
        Ok(
            MorphismInstance::Specialize {
                σ: vec![
                    ( 1,  dict.parse("test").expect("parse") )
                ].into_iter().collect(),
                m: Box::new(
                    MorphismInstance::Chain {
                        path: vec![
                            MorphismInstance::Specialize {
                                σ: vec![
                                    ( 0,  dict.parse("Y").expect("parse") )
                                ].into_iter().collect(),
                                m: Box::new(MorphismInstance::Primitive { m: DummyMorphism(MorphismType {
                                    bounds: Vec::new(),
                                    src_type: dict.parse("T ~ A").expect("parse"),
                                    dst_type: dict.parse("T ~ <B X> ~ U").expect("parse")
                                }) }) },
                            MorphismInstance::Primitive { m: DummyMorphism(MorphismType {
                                    bounds: Vec::new(),
                                    src_type: dict.parse("T ~ <B Y> ~ U").expect("parse"),
                                    dst_type: dict.parse("T ~ <B Y> ~ V").expect("parse")
                                }) }
                        ]
                    }
                )
            }
        )
    );
}


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
