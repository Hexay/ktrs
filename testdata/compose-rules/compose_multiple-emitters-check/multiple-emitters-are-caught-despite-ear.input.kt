@Composable
fun Something() {
    Text("1")
    if (x) {
        Text("2")
        return
    }
}
@Composable
fun Something() {
    if (x) {
        Text("1")
        Text("2")
        return
    }
}