@Composable
fun MyComposable() {
    val something = remember { movableContentWithReceiverOf { Text("X") } }
}