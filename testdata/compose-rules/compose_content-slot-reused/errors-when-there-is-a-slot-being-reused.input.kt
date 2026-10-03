@Composable
fun A(text: String, content: @Composable () -> Unit) {
    if (x) content() else content()
}
@Composable
fun B(text: String, content: @Composable () -> Unit) {
    when {
        x -> content()
        else -> content()
    }
}
@Composable
fun C(text: String, content: @Composable () -> Unit) {
    potato?.let { content() } ?: content()
}
@Composable
fun D(text: String, content: Potato) {
    potato?.let { content() } ?: content()
}
@Composable
fun E(text: String, content: @Composable () -> Unit) {
    val content1 = remember { movableContentOf { content() } }
    val content2 = remember { movableContentOf { content() } }
}