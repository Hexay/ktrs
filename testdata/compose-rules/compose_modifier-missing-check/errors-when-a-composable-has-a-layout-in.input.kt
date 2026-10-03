@Composable
fun Something() {
    Row {
    }
}
@Composable
fun Something() {
    Column(modifier = Modifier.fillMaxSize()) {
    }
}
@Composable
fun Something(): Unit {
    SomethingElse {
        Box(modifier = Modifier.fillMaxSize()) {
        }
    }
}
@Composable
fun Something(): Unit {
    SomethingElse {
        Box(Modifier.fillMaxSize()) {
        }
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    Row {
        Text("Hi!")
    }
}
@Composable
fun Something() {
    Column(modifier = BananaModifier.fillMaxSize()) {
    }
}