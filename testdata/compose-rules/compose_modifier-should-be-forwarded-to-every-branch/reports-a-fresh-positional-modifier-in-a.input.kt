@Composable
fun Content(state: Int, modifier: Modifier = Modifier) {
    if (state == 0) {
        LoadedContent("ready", modifier.padding(8.dp))
    } else if (state == 1) {
        LoadingContent("loading", Modifier)
    } else {
        LoadedContent("ready", modifier = modifier)
    }
}
@Composable
fun LoadedContent(label: String, modifier: Modifier = Modifier) {}
@Composable
fun LoadingContent(label: String, rootModifier: Modifier = Modifier) {}