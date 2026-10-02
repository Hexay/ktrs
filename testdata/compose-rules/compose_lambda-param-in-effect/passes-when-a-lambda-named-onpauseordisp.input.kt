@Composable
fun Something(onDispose: () -> Unit) {
    val latestOnDispose by rememberUpdatedState(onDispose)
    LifecycleResumeEffect(Unit) {
        onPauseOrDispose(latestOnDispose)
    }
}