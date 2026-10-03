@Composable
fun Something(modifier: Modifier) {
    Row(modifier) {
        val x = @Composable { modifier: Modifier ->
            SomethingElse(modifier)
        }
    }
}
@Composable
fun Something(modifier: Modifier) {
    Row(modifier) {
        val x = @Composable { (modifier, _) ->
            SomethingElse(modifier)
        }
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier) {
        Bleh { modifier -> Potato(modifier) }
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        Bleh { modifier: Modifier -> Row(modifier = Modifier.then(modifier)) }
    }
}
@Composable
fun Something(modifier: Modifier) {
    Row(modifier) {
        Slot { modifier: Modifier ->
            val local = modifier.padding(8.dp)
            Row(modifier = Modifier.then(local)) {}
        }
    }
}
@Composable
fun Something(modifier: Modifier) {
    Row(modifier) {
        Slot { (modifier, _) ->
            val local = modifier.padding(8.dp)
            Row(modifier = Modifier.then(local)) {}
        }
    }
}