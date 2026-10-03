@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        ChildThatReusesModifier(modifier = modifier.fillMaxWidth())
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        val newModifier = modifier.fillMaxWidth()
        ChildThatReusesModifier(modifier = newModifier)
    }
}
@Composable
fun Something(modifier: Modifier) {
    val newModifier = modifier.fillMaxWidth()
    Column(modifier = modifier) {
        ChildThatReusesModifier(modifier = newModifier)
    }
}
@Composable
fun Something(modifier: Modifier, otherModifier: Modifier) {
    Column(modifier = modifier) {
        val newModifier = modifier.fillMaxWidth()
        val newModifier2 = otherModifier.fillMaxWidth()
        ChildThatReusesModifier(modifier = newModifier)
        ChildThatReusesModifier(modifier = otherModifier)
        ChildThatReusesModifier(modifier = newModifier2)
    }
}