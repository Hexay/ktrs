@Composable
fun Something() {
    Text("Hi")
    Text("Hola")
}
@Composable
fun Something() {
    Spacer()
    Text("Hola")
}
@Composable
fun Something(title: String?, subtitle: String?) {
    title?.let { Text(title) }
    subtitle?.let { Text(subtitle) }
}
@Composable
fun Something(title: String?, subtitle: String?) {
    with(title) { Text(this) }
    with(subtitle) { Text(this) }
}