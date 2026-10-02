@Composable
fun TooDeepA() {
    Box { Box { Box { Box { Text("a") } } } }
}
@Composable
fun ShallowB() {
    Box { Text("b") }
}
@Composable
fun TooDeepC() {
    Column { Row { Box { Box { Text("c") } } } }
}