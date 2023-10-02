# lib-laddertypes

Rust Implementation of Ladder-Types (parsing, unification, rewriting, etc) 
<hr/>

## Ladder Types

*Ladder Types* provide a way to distinguish between multiple *concrete
representations* of the same *abstract / conceptual type*.

The intuition is to capture a 'represented-as' relation in the
structure of a type term. Formally, we introduce a new type
constructor, called the *ladder type*, written as `T1 ~ T2`, where
`T1` and `T2` are types. The term `T1 ~ T2` can be read by `T1 is
represented as T2`.



This can be particularly useful in contexts where no standard
representational forms exist, e.g. in the unix shell where many
different tools & utilities come into contact, each making use of
potentially different representational forms, depending on the
application domain.

Trying to introduce a global type system at IPC level would result in
naming conflicts: Abstract concepts like 'natural number' do not have
any standard representational form, as it is the case with other more
contained languages. With a simple type name like this it is not clear
which of the many equivalent representations is required, e.g. for an
*integer number* variations over radices, endianness, digit encoding
etc. exist.

Usually, many layers of abstraction are built ontop of each other,
where higher-level data types are encoded into lower-level data types,
forming a chain of embeddings from concept to `rock bottom' of byte
streams.


#### Example
The following type describes a colon-separated sequence of timepoints,
each represented as unix-timestamp written as decimal number in
big-endian, encoded as UTF-8 string.

```
<Seq Timepoint
     ~<TimeSince UnixEpoch>
     ~<Duration Seconds>
     ~ℕ
     ~<PosInt 10 BigEndian>
     ~<Seq <Digit 10>~Char>>
~<SepSeq Char ':'>
~<Seq Char>
~UTF-8
~<Seq Byte>
```

An object that fits the format described by this type could look like
this:

```
1696093021:1696093039:1528324679:1539892301:1638141920:1688010253
```

## How to use this crate

```rust
use laddertypes::*;

let mut dict = TypeDict::new();

let t1 = dict.parse("<A B~X C>").expect("couldnt parse typeterm");
let t2 = dict.parse("<<A B~X> C>").expect("couldnt parse typeterm");

assert_eq!( t1.clone().curry(), t2 );
assert_eq!( t1, t2.clone().decurry() );
```

### License
[GPLv3](COPYING)

