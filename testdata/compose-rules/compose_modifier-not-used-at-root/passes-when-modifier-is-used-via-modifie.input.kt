@Composable
fun Something(modifier: Modifier = Modifier) {
    Column(modifier = modifier) {
        Slot { modifier: Modifier ->
            Row(modifier = Modifier.then(modifier)) {}
        }
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    Column(modifier = modifier) {
        Slot { (modifier, _) ->
            val local = modifier.padding(8.dp)
            Row(modifier = Modifier.then(local)) {}
        }
    }
}