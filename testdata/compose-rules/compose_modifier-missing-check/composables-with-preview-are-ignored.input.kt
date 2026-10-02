@Preview
@Composable
fun Something() {
    Row {
    }
}
@PreviewScreenSizes
@Composable
fun Something() {
    Row {
    }
}
@Preview
@Composable
fun Something() {
    Column(modifier = Modifier.fillMaxSize()) {
    }
}
@Preview
@Composable
fun Something(): Unit {
    SomethingElse {
        Box(modifier = Modifier.fillMaxSize()) {
        }
    }
}