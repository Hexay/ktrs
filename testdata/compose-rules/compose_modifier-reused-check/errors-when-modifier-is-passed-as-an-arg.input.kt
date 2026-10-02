@Composable
fun Something(modifier: Modifier) {
    Row(modifier = Modifier.then(modifier)) {
        Column(modifier = modifier) {}
    }
}
@Composable
fun Something(modifier: Modifier) {
    Row(modifier = Modifier.fillMaxWidth().then(modifier)) {}
    Column(modifier = modifier) {}
}