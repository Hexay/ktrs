class Foo1<A, B> {}
class Foo2<
    A,
    B // The comma before the comment should be removed without removing the comment itself
> {}
class Foo3<
    A,
    B /* The comma before the comment should be removed without removing the comment itself */
> {}