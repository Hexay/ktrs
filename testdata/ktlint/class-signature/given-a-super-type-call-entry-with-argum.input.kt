// Max line length marker:                            #
// Note that the super type call does not fit the line after it has been wrapped
// ...Foo(a: Any, b: Any, c: Any) :
//  FooBar(a, "longggggggggggggggggggggggggggggggggggg")
class Foo(a: Any, b: Any, c: Any) : FooBar(a, "longggggggggggggggggggggggggggggggggggg") {
    // body
}