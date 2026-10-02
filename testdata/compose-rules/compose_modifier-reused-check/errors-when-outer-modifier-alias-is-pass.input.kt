@Composable
fun Something(modifier: Modifier) {
    val rootModifier = modifier
    Column(modifier = rootModifier) {
        Slot { modifier: Modifier ->
            Child(modifier = rootModifier, extra = modifier)
        }
    }
}