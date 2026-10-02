@Composable
fun MyDialog() {
    AlertDialog(
        onDismissRequest = { /*TODO*/ },
        buttons = { Text(text = "Button") },
        text = { Text(text = "Body") },
    )
    PotatoDialog(
        onDismissRequest = { /*TODO*/ },
        buttons = { Text(text = "Button") },
        text = { Text(text = "Body") },
    )
}