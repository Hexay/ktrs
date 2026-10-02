@Composable
fun A(text: String, content: @Composable () -> String) {
    if (x) content() else content()
}
@Composable
fun B(text: String, content: @Composable (() -> String)?) {
    if (x) content() else content()
}
