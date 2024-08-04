use crate::dict::TypeDict;

#[test]
fn test_param_normalize() {
    let mut dict = TypeDict::new();

    assert_eq!(
        dict.parse("A~B~C").expect("parse error"),
        dict.parse("A~B~C").expect("parse error").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A B>~C").expect("parse error"),
        dict.parse("<A B>~C").expect("parse error").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A B~C>").expect("parse error"),
        dict.parse("<A B>~<A C>").expect("parse error").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A B~C D~E>").expect("parse error"),
        dict.parse("<A B D>~<A C D>~<A C E>").expect("parse errror").param_normalize(),
    );

    assert_eq!(
        dict.parse("<Seq <Digit 10>~Char>").expect("parse error"),
        dict.parse("<Seq <Digit 10>>~<Seq Char>").expect("parse errror").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A <B C~D~E> F~G H H>").expect("parse error"),
        dict.parse("<A <B C> F H H>
                   ~<A <B D> F H H>
                   ~<A <B E> F H H>
                   ~<A <B E> G H H>").expect("parse errror")
               .param_normalize(),
    );
}

