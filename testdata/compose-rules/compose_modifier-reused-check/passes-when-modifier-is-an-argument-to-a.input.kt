@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        Child(model = SomePipeline.then(modifier))
    }
}