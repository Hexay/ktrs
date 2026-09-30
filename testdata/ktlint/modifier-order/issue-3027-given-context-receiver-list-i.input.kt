// context receiver
@Bar context(Foo) private fun bar() {}
// context parameter
@Bar context(_: Foo) private fun bar() {}