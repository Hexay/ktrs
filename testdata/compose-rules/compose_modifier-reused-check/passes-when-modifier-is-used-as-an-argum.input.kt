@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        Icon(painter = PainterFactory.create(modifier))
    }
}