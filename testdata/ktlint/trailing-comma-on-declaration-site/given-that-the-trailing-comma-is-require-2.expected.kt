class Foo1<A, B> {}
class Foo2<
    A,
    B, // The comma should be inserted before the comment
> {}
class Foo3<
    A,
    B, /* The comma should be inserted before the comment */
> {}