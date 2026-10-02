@Composable
fun Something(onDispose: () -> Unit) {
    val latestOnDispose by rememberUpdatedState(onDispose)
    DisposableEffect(Unit) {
        onDispose(latestOnDispose)
    }
}