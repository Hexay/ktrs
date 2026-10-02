context(columnScope: ColumnScope)
@Composable
fun Something() {
    Text("Hi")
    Text("Hola")
}
context(rowScope: RowScope)
@Composable
fun Something() {
    Spacer()
    Text("Hola")
}