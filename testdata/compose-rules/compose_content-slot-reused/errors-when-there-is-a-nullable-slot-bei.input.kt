@Composable
fun A(text: String, content: @Composable (() -> Unit)? = null) {
    if (x) content?.invoke() else content?.invoke()
}
@Composable
fun B(text: String, content: @Composable (() -> Unit)? = null) {
    when {
        x -> content?.invoke()
        else -> content?.invoke()
    }
}
@Composable
fun C(text: String, content: @Composable (() -> Unit)? = null) {
    val content1 = remember { movableContentOf { content?.invoke() } }
    val content2 = remember { movableContentOf { content?.invoke() } }
}
@Composable
fun D(content: Potato? = null) {
    if (x) content?.invoke() else content?.invoke()
}