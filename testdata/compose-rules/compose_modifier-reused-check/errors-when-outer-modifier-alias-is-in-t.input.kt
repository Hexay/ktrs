@Composable
fun Something(modifier: Modifier) {
    val rootModifier = modifier
    Column(modifier = rootModifier) {
        Slot { modifier: Modifier ->
            Row(modifier = modifier.then(rootModifier)) {}
        }
    }
}