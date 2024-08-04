use {
    crate::{dict::*, morphism::*, parser::*}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_morphism_path() {
    let mut dict = BimapTypeDict::new();
    let mut base = MorphismBase::<u64>::new( dict.add_typename("Seq".into()) );

    dict.add_varname("Radix".into());
    dict.add_varname("SrcRadix".into());
    dict.add_varname("DstRadix".into());

    base.add_morphism(
        MorphismType{
            src_type: dict.parse("<Digit Radix> ~ Char").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap()
        },
        11
    );
    base.add_morphism(
        MorphismType{
            src_type: dict.parse("<Digit Radix> ~ ℤ_2^64 ~ machine.UInt64").unwrap(),
            dst_type: dict.parse("<Digit Radix> ~ Char").unwrap()
        },
        22
    );
    base.add_morphism(
        MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        },
        333
    );
    base.add_morphism(
        MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt Radix LittleEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt Radix BigEndian> ~ <Seq <Digit Radix>~ℤ_2^64~machine.UInt64>").unwrap()
        },
        444
    );
    base.add_morphism(
        MorphismType{
            src_type: dict.parse("ℕ ~ <PosInt SrcRadix LittleEndian> ~ <Seq <Digit SrcRadix>~ℤ_2^64~machine.UInt64>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt DstRadix LittleEndian> ~ <Seq <Digit DstRadix>~ℤ_2^64~machine.UInt64>").unwrap()
        },
        555
    );


    assert_eq!(
        base.find_morphism_path(MorphismType {
            src_type: dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap(),
            dst_type: dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap()
        }),
        Some(
            vec![
                dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ Char>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 10 BigEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 10 LittleEndian> ~ <Seq <Digit 10> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 LittleEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ ℤ_2^64 ~ machine.UInt64>").unwrap().normalize(),
                dict.parse("ℕ ~ <PosInt 16 BigEndian> ~ <Seq <Digit 16> ~ Char>").unwrap().normalize(),
            ]
        )
    );
}
