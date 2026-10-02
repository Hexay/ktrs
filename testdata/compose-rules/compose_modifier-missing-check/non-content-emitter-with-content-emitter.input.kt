@Composable
fun MyDialog() {
    Text(text = "Unicorn")

    AlertDialog(
        onDismissRequest = { /*TODO*/ },
        buttons = { Text(text = "Button") },
        text = { Text(text = "Body") },
    )
}