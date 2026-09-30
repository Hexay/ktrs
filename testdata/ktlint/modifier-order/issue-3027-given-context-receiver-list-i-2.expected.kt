// context receiver
@Bar context(Foo) private fun bar1() {}
@Bar context(Foo) private fun bar2() {}
// context parameter
@Bar context(_: Foo) private fun bar1() {}
@Bar context(_: Foo) private fun bar2() {}