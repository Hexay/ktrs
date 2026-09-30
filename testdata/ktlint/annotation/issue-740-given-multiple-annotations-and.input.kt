annotation class A
annotation class B

@A // comment
@B
fun foo1() {}

@A
@B // comment
fun foo2() {}