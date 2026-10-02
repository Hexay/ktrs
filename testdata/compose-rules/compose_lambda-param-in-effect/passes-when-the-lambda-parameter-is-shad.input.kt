@Composable
fun Something(onClick: () -> Unit) {
    val onClick by rememberUpdatedState(onClick)
    LaunchedEffect(Unit) {
        onClick()
    }
}
@Composable
fun Something(onClick: () -> Unit) {
    val (onClick, bleh) = something()
    LaunchedEffect(Unit) {
        onClick()
    }
}