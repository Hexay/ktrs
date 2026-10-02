@Composable
fun Something() {
    androidx.compose.material.TopAppBar(title = { Text("boo") })
    Icon(imageVector = androidx.compose.material.icons.Icons.Arrow, contentDescription = null)
}