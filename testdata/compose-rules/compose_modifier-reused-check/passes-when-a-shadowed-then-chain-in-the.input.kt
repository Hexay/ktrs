@Composable
fun Something(modifier: Modifier = Modifier) {
    val rootModifier = modifier
    Column {
        Slot { modifier: Modifier ->
            Row(modifier = Modifier.then(modifier)) {}
            Box(modifier = rootModifier) {}
        }
    }
}