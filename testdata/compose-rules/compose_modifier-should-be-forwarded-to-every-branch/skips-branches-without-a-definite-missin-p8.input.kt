@Composable
fun Content(state: Int, modifier: Modifier = Modifier) {
    when (val modifier = otherModifier) { Modifier -> LoadingContent(); else -> LoadedContent(modifier) }
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