@Composable
fun MyComposable() = Text("bleh")

val composable: Something
    @Composable get() { }

val composable: Something
    @Composable get() = OtherComposable()

val whatever = @Composable { }