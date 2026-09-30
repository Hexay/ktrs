@Target(AnnotationTarget.TYPE)
annotation class Foo
val foo1: List<@Foo String> = emptyList()
val foo2 = emptyList<@Foo String>()
val foo3: Map<@Foo String, @Foo String> = emptyMap()
val foo4 = emptyMap<@Foo String, @Foo String>()