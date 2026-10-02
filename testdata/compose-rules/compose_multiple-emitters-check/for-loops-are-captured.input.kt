@Composable
fun MultipleContent(texts: List<String>, modifier: Modifier = Modifier) {
    for (text in texts) {
        Text(text)
    }
}
@Composable
fun MultipleContent(otherTexts: List<String>, modifier: Modifier = Modifier) {
    Text("text 1")
    for (otherText in otherTexts) {
        Text(otherText)
    }
}