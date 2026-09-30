class Foo(@Path("fooId") val fooId: String)
class Bar(
    @NotNull("fooId") val fooId: String,
    @NotNull("bar") bar: String
)