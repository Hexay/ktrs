@Composable
fun Something() {
    Whatever(modifier = Modifier.fillMaxSize()) {
    }
}
@Composable
fun Something(): Unit {
    SomethingElse {
        Whatever(modifier = Modifier.fillMaxSize()) {
        }
    }
}