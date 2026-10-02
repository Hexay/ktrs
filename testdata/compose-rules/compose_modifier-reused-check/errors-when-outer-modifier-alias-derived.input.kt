@Composable
fun Something(modifier: Modifier) {
    val rootModifier = modifier
    Column(modifier = rootModifier) {
        Slot { modifier: Modifier ->
            val childModifier = rootModifier.padding(8.dp)
            Row(modifier = childModifier) {}
        }
    }
}