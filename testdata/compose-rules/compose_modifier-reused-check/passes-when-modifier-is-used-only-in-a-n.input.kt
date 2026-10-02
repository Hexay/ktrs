@Composable
fun Something(modifier: Modifier = Modifier) {
    @Composable fun Local(modifier: Modifier) {
        val local = modifier.padding(8.dp)
        Row(modifier = Modifier.then(local)) {}
    }
    Column(modifier = modifier) {}
}