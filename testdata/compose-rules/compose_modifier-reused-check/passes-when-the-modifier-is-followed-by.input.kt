@Composable
fun Something(modifier: Modifier) {
    if (LocalInspectionMode.current) {
        DebugPlaceholder(modifier = modifier)
        return
    }
    Box(modifier = modifier) {
    }
}