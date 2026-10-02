@Composable
fun MyComposable() {
    val something = retain { movableContentWithReceiverOf { Text("X") } }
}