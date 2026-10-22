@Composable
fun Content(state: Int, modifier: Modifier = Modifier) {
    if (state == 0) LoadingContent() else LoadedContent()
}
@Composable
fun LoadedContent(modifier: Modifier = Modifier) {}
@Composable
fun LoadingContent(modifier: Modifier = Modifier) {}
@Composable
fun EmptyContent() {}
@Composable
fun MixedContent(label: String, modifier: Modifier = Modifier) {}
@Composable
fun OverloadedContent(modifier: Modifier = Modifier) {}
@Composable
fun OverloadedContent(label: String = "", modifier: Modifier = Modifier) {}