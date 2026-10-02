@Composable
fun Something1(modifier: Modifier = Modifier) {
    Something2(
        modifier = Modifier.clickable { }.shadow(8.dp, RoundedCornerShape(8.dp))
    )
    Something3(
        modifier = modifier.clickable { }.shadow(elevation = 4.dp, shape = CircleShape)
    )
    Something4(
        modifier.clickable { }.then(if (x) shadow(8.dp, MyShape) else Modifier)
    )
    Something5(
        modifier.clickable { }.then(if (x) Modifier.shadow(8.dp, MyShape) else Modifier)
    )
    Something6(
        modifier.clickable { }.then(if (x) Modifier.shadow(8.dp, MyShape).padding(4.dp) else Modifier)
    )
    Something7(
        modifier = modifier.clickable { }.shadow(8.dp, shape)
    )
}