import kotlin.reflect.KClass

@Foo(
    values = [
        Foo::class,
        Foo::class,
    ],
)
annotation class Foo(val values: Array<KClass<*>>)