use crate::{dict::*, heuristic::*, morphism::*, parser::*, Context};

#[test]
fn test_heuristic() {
    let mut dict = Context::new();

    assert_eq!(
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: dict.parse("A").expect("parse"),
            dst_type: dict.parse("A").expect("parse")
        }.estimated_cost(),
        0
    );

    assert_eq!(
        MorphismType {
            Γ: Vec::new(),
            bounds: Vec::new(),
            src_type: dict.parse("<Digit 10> ~ Char ~ Ascii ~ native.UInt8").expect("parse"),
            dst_type: dict.parse("<Digit 16> ~ native.UInt8").expect("parse")
        }.estimated_cost(),
        40
    );
}
