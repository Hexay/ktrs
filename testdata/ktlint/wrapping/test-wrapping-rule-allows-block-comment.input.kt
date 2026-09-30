interface Foo1 {}
interface Foo2 {}
interface Foo3 {}
class Bar :
    Foo1, /* this comment should be legal */
    Foo2,/* this comment should be legal */
    Foo3 {
}