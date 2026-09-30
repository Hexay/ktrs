@Suppress("Something") class FooBar1 {
    @Suppress("Something") var foo: String
    @Suppress("Something") fun bar() {}
}
@Suppress("Something") /* some comment */ class FooBar2 {
    @Suppress("Something") /* some comment */ var foo: String
    @Suppress("Something") /* some comment */ fun bar() {}
}
@Suppress("Something") /** some comment */ class FooBar3 {
    @Suppress("Something") /** some comment */ var foo: String
    @Suppress("Something") /** some comment */ fun bar() {}
}