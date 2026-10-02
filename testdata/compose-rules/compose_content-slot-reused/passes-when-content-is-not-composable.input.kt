@Composable
fun A(content: () -> Unit, text: String) {
    if (x) content() else content()
}
fun B(content: Plum, text: String) {
    if (x) content() else content()
}
@Composable
fun C(content: (() -> Unit)? = null) {
    if (x) content?.invoke() else content?.invoke()
}
@Composable
fun D(content: Plum? = null) {
    if (x) content?.invoke() else content?.invoke()
}