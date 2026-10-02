@Composable
fun MyComposable() {
    val something = remember { movableContentOf { Text("X") } }
}