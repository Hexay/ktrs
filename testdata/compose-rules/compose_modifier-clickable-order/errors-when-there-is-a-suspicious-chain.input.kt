@Composable
fun Something1(modifier: Modifier = Modifier, bananaModifier: Modifier = Modifier) {
    Something2(
        modifier = Modifier.clickable { }.clip(shape = RoundedCornerShape(8.dp))
    )
    Something3(
        modifier = modifier.clickable { }.clip(CircleShape())
    )
    Something4(
        Modifier.clickable { }.clip(MyShape)
    )
    Something5(
        modifier = Modifier.clip(CircleShape).clickable { }.background(MyShape)
    )
    Something6(
        modifier.clickable { }.then(if (x) border(TurdShape) else Modifier)
    )
    Something7(
        modifier = bananaModifier.clickable { }.clip(shape = RoundedCornerShape(8.dp))
    )
    Something8(
        modifier = bananaModifier.clickable { }.clip(Potato)
    )
    Something9(
        modifier = bananaModifier.clickable { }.background(MaterialTheme.shapes.large)
    )
}