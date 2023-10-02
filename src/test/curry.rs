use {
    crate::{term::*, dict::*, parser::*},
    std::str::FromStr
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\

#[test]
fn test_curry() {
    let mut dict = TypeDict::new();

    assert_eq!(
        dict.parse("<A B C>").unwrap().curry(),
        dict.parse("<<A B> C>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C D>").unwrap().curry(),
        dict.parse("<<<A B> C> D>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C D E F G H I J K>").unwrap().curry(),
        dict.parse("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap()
    );

    assert_eq!(
        dict.parse("<A~X B C>").unwrap().curry(),
        dict.parse("<<A~X B> C>").unwrap()
    );
    assert_eq!(
        dict.parse("<A B C~Y~Z> ~ K").unwrap().curry(),
        dict.parse("< <A B> C~Y~Z > ~ K").unwrap()
    );
}

#[test]
fn test_decurry() {
    let mut dict = TypeDict::new();

    assert_eq!(
        dict.parse("<<A B> C>").unwrap().decurry(),
        dict.parse("<A B C>").unwrap()
    );
    assert_eq!(
        dict.parse("<<<A B> C> D>").unwrap().decurry(),
        dict.parse("<A B C D>").unwrap(),
    );
    assert_eq!(
        dict.parse("<<<<<<<<<<A B> C> D> E> F> G> H> I> J> K>").unwrap().decurry(),
        dict.parse("<A B C D E F G H I J K>").unwrap()
    );
    
    assert_eq!(
        dict.parse("<<A~X B> C>").unwrap().decurry(),
        dict.parse("<A~X B C>").unwrap()
    );
    assert_eq!(
        dict.parse("<<A~X B> C~Y>~K").unwrap().decurry(),
        dict.parse("<A~X B C~Y> ~K").unwrap()
    );
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>\\
