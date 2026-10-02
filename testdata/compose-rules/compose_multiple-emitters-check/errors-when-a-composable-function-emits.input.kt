@Composable
fun A() {
    if (something) {
        Text("1")
        Text("2")
    } else {
        Text("1")
        Text("2")
    }
}
@Composable
fun B() {
    if (something) {
        Text("1")
        Text("2")
    } else {
        Text("1")
    }
}
@Composable
fun C() {
    if (something) {
        Text("1")
    } else {
        Text("1")
        Text("2")
    }
}
@Composable
fun D() {
    if (something) {
        Text("1")
    }
    Text("2")
}