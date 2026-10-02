@Composable
fun Something(modifier: Modifier) {
    Row(modifier) {
        SomethingElse(modifier)
    }
}
@Composable
fun Something(modifier: Modifier): Int {
    Column(modifier = modifier) {
        SomethingElse(modifier = Modifier)
        SomethingDifferent(modifier = modifier)
    }
}
@Composable
fun BoxScope.Something(modifier: Modifier) {
    Column(modifier = modifier) {
        SomethingDifferent()
    }
    SomethingElse(modifier = modifier)
    SomethingElse(modifier = modifier.padding12())
}
@Composable
fun Something(myMod: Modifier) {
    Column {
        SomethingElse(myMod)
        SomethingElse(myMod)
    }
}
@Composable
fun FoundThisOneInTheWild(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .size(AvatarSize.Default.size)
            .clip(CircleShape)
            .then(colorModifier)
    ) {
        Box(
            modifier = modifier.padding(spacesBorderWidth)
        )
    }
}
@Composable
fun Something(modifier: Modifier, otherModifier: Modifier): Int {
    Column(modifier = modifier) {
        SomethingElse(modifier = otherModifier)
        SomethingDifferent(modifier = otherModifier)
    }
}