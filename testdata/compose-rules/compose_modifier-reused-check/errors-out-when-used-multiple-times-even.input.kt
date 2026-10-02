@Composable
fun Something(modifier: Modifier) {
    Box(modifier = modifier) {
    }
    if (LocalInspectionMode.current) {
        DebugPlaceholder(modifier = modifier)
        return
    }
}
@Composable
fun Something(modifier: Modifier) {
    if (LocalInspectionMode.current) {
        Text("bleh", modifier = modifier)
        DebugPlaceholder(modifier = modifier)
        return
    }
    Box(modifier = modifier) {}
}
@Composable
fun Something(modifier: Modifier) {
    if (LocalInspectionMode.current) {
        Text("bleh", modifier = modifier)
        if (x) {
            DebugPlaceholder(modifier = modifier)
            return
        }
    }
}