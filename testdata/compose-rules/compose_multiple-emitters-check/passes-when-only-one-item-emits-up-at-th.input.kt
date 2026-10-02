@Composable
fun Something() {
    val something = rememberWhatever()
    Column {
        Text("Hi")
        Text("Hola")
    }
    LaunchedEffect(Unit) {
    }
}