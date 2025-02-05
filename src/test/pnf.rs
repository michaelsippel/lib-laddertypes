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
        dict.parse("<A~Y B>").expect("parse error"),
        dict.parse("<A B>~<Y B>").expect("parse error").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A B~C D~E>").expect("parse error"),
        dict.parse("<A B D>~<A C D>~<A C E>").expect("parse errror").param_normalize(),
    );

    assert_eq!(
        dict.parse("<A~X B~C D~E>").expect("parse error"),
        dict.parse("<A B D>~<A B~C E>~<X C E>").expect("parse errror").param_normalize(),
    );

    assert_eq!(
        dict.parse("<Seq <Digit 10>~Char>").expect("parse error"),
        dict.parse("<Seq <Digit 10>>~<Seq Char>").expect("parse errror").param_normalize(),
    );

    assert_eq!(
        dict.parse("<Seq Char> ~ <<ValueDelim '\\0'> Char> ~ <<ValueDelim '\\0'> Ascii~x86.UInt8>").expect("parse error").param_normalize(),
        dict.parse("<Seq~<ValueDelim '\\0'> Char~Ascii~x86.UInt8>").expect("parse error")
    );
    assert_eq!(
        dict.parse("<Seq Char~Ascii> ~ <<ValueDelim '\\0'> Char~Ascii> ~ <<ValueDelim '\\0'> x86.UInt8>").expect("parse error").param_normalize(),
        dict.parse("<Seq~<ValueDelim '\\0'> Char~Ascii~x86.UInt8>").expect("parse error")
    );

    assert_eq!(
        dict.parse("<A~Y <B C~D~E> F H H>").expect("parse error"),
        dict.parse("<A <B C> F H H>
                   ~<A <B D> F H H>
                   ~<A~Y <B E> F H H>").expect("parse errror")
               .param_normalize(),
    );
}

