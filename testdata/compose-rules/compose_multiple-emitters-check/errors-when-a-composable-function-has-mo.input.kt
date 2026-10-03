@Composable
fun Something1() {
    Something2()
}
@Composable
fun Something2() {
    Text("Hola")
    Something3()
}
@Composable
fun Something3() {
    Text("Hi")
}
@Composable
fun Something4() {
    Banana()
}
@Composable
fun Something5() {
    Something3()
    Something4()
}