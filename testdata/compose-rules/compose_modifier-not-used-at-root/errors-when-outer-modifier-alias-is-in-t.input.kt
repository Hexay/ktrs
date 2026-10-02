@Composable
fun Something(modifier: Modifier = Modifier) {
    val rootModifier = modifier
    Column {
        Slot { modifier: Modifier ->
            Child(modifier = modifier.then(rootModifier))
        }
    }
}