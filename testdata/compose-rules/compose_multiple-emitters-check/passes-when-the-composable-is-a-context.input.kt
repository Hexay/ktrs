context(ColumnScope)
@Composable
fun Something() {
    Text("Hi")
    Text("Hola")
}
context(RowScope)
@Composable
fun Something() {
    Spacer()
    Text("Hola")
}