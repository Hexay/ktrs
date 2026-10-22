@Composable
fun Content(state: Int, modifier: Modifier = Modifier) {
    when (state) {
        0 -> LoadedContent(modifier)
        else -> LoadingContent()
    }
}
@Composable
fun LoadedContent(modifier: Modifier = Modifier) {}
@Composable
fun LoadingContent(modifier: Modifier = Modifier) {}