@Composable
fun Something(modifier: Modifier) {
    val rootModifier = modifier
    Column(modifier = rootModifier) {
        Slot { modifier: Modifier ->
            Row(modifier = rootModifier.then(modifier)) {}
        }
    }
}