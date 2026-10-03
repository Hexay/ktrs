@Composable
fun Something(onClick: () -> Unit) {
    LaunchedEffect(Unit) {
        onClick()
    }
}
@Composable
fun Something(onClick: MyLambda) {
    DisposableEffect(Unit) {
        onClick()
    }
}
fun interface MyLambda2 {
    fun create()
}
@Composable
fun Something(onClick: MyLambda2) {
    LaunchedEffect(Unit) {
        onClick()
    }
}