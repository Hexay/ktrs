@Composable
fun Something(onDispose: () -> Unit) {
    val latestOnDispose by rememberUpdatedState(onDispose)
    LifecycleStartEffect(Unit) {
        onStopOrDispose(latestOnDispose)
    }
}