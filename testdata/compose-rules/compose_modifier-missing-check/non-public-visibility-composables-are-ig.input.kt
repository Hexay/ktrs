@Composable
private fun Something() {
    Row {
    }
}
@Composable
protected fun Something() {
    Column(modifier = Modifier.fillMaxSize()) {
    }
}
@Composable
internal fun Something() {
    SomethingElse {
        Box(modifier = Modifier.fillMaxSize()) {
        }
    }
}
@Composable
private fun Something() {
    Whatever(modifier = Modifier.fillMaxSize()) {
    }
}