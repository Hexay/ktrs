@Composable
fun Something1() {
    Something2(
        modifier = Modifier.clip(RoundedCornerShape(8.dp)).background(shape = Circle()).clickable { }
    )
    Something2(
        modifier = Modifier.clip(shape = Whatever).background().clickable { }
    )
}