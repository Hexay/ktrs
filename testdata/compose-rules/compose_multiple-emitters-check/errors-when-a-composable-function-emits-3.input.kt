@Composable
fun A() {
    text?.let {
        Text("1")
        Text("2")
    } ?: run {
        Text("1")
        Text("2")
    }
}
@Composable
fun B() {
    text?.let {
        Text("1")
        Text("2")
    } ?: Text("1")
}
@Composable
fun C() {
    text?.let {
        Text("1")
    } ?: run {
        Text("1")
        Text("2")
    }
}
@Composable
fun D() {
    text?.let { Text("1") }
    Text("2")
}