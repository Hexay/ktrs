@Composable
fun Something1() {
    Something2(
        modifier = Modifier.shadow(8.dp, RoundedCornerShape(8.dp)).clickable { }
    )
    Something3(
        modifier = Modifier.clickable { }.shadow(8.dp, MyShape, clip = false)
    )
    Something4(
        modifier = Modifier.clickable { }.shadow(8.dp, MyShape, false)
    )
    Something5(
        modifier = Modifier.clickable { }.shadow(0.dp, MyShape)
    )
    Something6(
        modifier = Modifier.clickable { }.shadow(elevation = 0.dp, shape = MyShape)
    )
    Something7(
        modifier = Modifier.clickable { }
            .then(if (x) run { something(Modifier.clip(CircleShape)); Modifier } else Modifier)
    )
}