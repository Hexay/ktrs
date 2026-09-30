// context receiver
context(Foo) @Bar private fun bar1() {}
private context(Foo) @Bar fun bar2() {}
// context parameter
context(_: Foo) @Bar private fun bar1() {}
private context(_: Foo) @Bar fun bar2() {}