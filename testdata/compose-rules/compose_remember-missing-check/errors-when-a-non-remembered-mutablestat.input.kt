@Composable
fun MyComposable() {
    val something = mutableStateOf("X")
}
@Composable
fun MyComposable(something: State<String> = mutableStateOf("X")) {
}