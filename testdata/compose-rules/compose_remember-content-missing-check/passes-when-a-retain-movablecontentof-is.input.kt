@Composable
fun MyComposable() {
    val something = retain { movableContentOf { Text("X") } }
}