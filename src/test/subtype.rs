use crate::{dict::BimapTypeDict, parser::*, unparser::*};
/*
#[test]
fn test_semantic_subtype() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        dict.parse_desugared("A~B~C").expect("parse error")
            .is_semantic_subtype_of(
                &dict.parse_desugared("A~B~C").expect("parse errror")
            ),
        Some((0, dict.parse_desugared("A~B~C").expect("parse errror")))
    );

    assert_eq!(
        dict.parse_desugared("A~B1~C1").expect("parse error")
            .is_semantic_subtype_of(
                &dict.parse_desugared("A~B2~C2").expect("parse errror")
            ),
        Some((0, dict.parse_desugared("A~B1~C1").expect("parse errror")))
    );

    assert_eq!(
        dict.parse_desugared("A~B~C1").expect("parse error")
            .is_semantic_subtype_of(
                &dict.parse_desugared("B~C2").expect("parse errror")
            ),
        Some((1, dict.parse_desugared("B~C1").expect("parse errror")))
    );
}

#[test]
fn test_syntactic_subtype() {
    let mut dict = BimapTypeDict::new();

    assert_eq!(
        dict.parse_desugared("A~B~C").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("A~B~C").expect("parse errror")
            ),
        Ok(0)
    );

    assert_eq!(
        dict.parse_desugared("A~B~C").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("B~C").expect("parse errror")
            ),
        Ok(1)
    );

    assert_eq!(
        dict.parse_desugared("A~B~C~D~E").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("C~D").expect("parse errror")
            ),
        Ok(2)
    );

    assert_eq!(
        dict.parse_desugared("A~B~C~D~E").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("C~G").expect("parse errror")
            ),
        Err((2,3))
    );

    assert_eq!(
        dict.parse_desugared("A~B~C~D~E").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("G~F~K").expect("parse errror")
            ),
        Err((0,0))
    );

    assert_eq!(
        dict.parse_desugared("<Duration Seconds>~ℕ").expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("ℕ").expect("parse errror")
            ),
        Ok(1)
    );

    assert_eq!(
        dict.parse_desugared("
<Duration Seconds>
~ℕ
~<PosInt 10 BigEndian>
~< Seq   <Digit 10> ~ Char >"
        ).expect("parse error")
            .is_syntactic_subtype_of(
                &dict.parse_desugared("<Seq Char>").expect("parse errror")
            ),
        Ok(4)
    );
}
*/
