// The property "__foo" in example below can be defined in an external dependency, which can not be changed. Even in case it is
// a (internal) dependency that can be changed, the violation should only be reported at the base property, but on the overrides
val fooBar =
    object : FooBar {
        override val __foo = "foo"
    }