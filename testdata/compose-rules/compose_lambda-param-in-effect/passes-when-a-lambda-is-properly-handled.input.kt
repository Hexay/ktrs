@Composable
fun Something(onClick: () -> Unit) {
    val latestOnClick by rememberUpdatedState(onClick)
    LaunchedEffect(Unit) {
        latestOnClick()
    }
}
@Composable
fun Something(onClick: () -> Unit) {
    DisposableEffect(onClick) {
        onClick()
    }
}