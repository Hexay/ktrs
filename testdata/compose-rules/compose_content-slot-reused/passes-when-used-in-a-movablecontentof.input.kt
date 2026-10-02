@Composable
fun A(content: @Composable () -> Unit, text: String) {
    val content = remember { movableContentOf { content() } }
    if (x) content() else content()
}