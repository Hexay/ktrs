@Composable
fun Something(onDispose: () -> Unit) {
    LaunchedEffect(Unit) {
        onDispose()
    }
}
@Composable
fun Something(onDispose: () -> Unit) {
    DisposableEffect(Unit) {
        onDispose(onDispose)
    }
}
@Composable
fun Something(onDispose: () -> Unit) {
    LifecycleStartEffect(Unit) {
        onStopOrDispose(onDispose)
    }
}
@Composable
fun Something(onDispose: () -> Unit) {
    LifecycleResumeEffect(Unit) {
        onPauseOrDispose(onDispose)
    }
}

// TODO ideally these would also be caught, but may require type resolution
@Composable
fun Something(onDispose: () -> Unit) {
    DisposableEffect(Unit) {
        onDispose { onDispose() }
    }
}
@Composable
fun Something(onDispose: (Int) -> Unit) {
    DisposableEffect(Unit) {
        onDispose(0)
        onDispose {}
    }
}