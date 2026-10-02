@Composable
fun MyComposable() {
    val something = derivedStateOf { "X" }
}
@Composable
fun MyComposable(something: State<String> = derivedStateOf { "X" }) {
}