@Composable
fun Content(modifier: Modifier) {
    Column(modifier = modifier) {
        Slot { modifier: Modifier ->
            Row(modifier = Modifier.then((modifier))) {}
        }
    }
}