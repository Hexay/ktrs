@Composable
fun Something(modifier: Modifier = Modifier) {
    Row {
        Text("Hi", modifier = modifier)
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    Potato(Modifier.fillMaxWidth()) {
        Text("Hi", modifier = modifier)
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    val poop = if (x) modifier else modifier.fillMaxWidth()
    Column {
        Text("Hi", modifier = poop)
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    if (paella.isWellDone()) {
        Column {
            Text("Yay", modifier)
        }
    } else {
        Row {
            Text("Oh no", modifier)
        }
    }
}
