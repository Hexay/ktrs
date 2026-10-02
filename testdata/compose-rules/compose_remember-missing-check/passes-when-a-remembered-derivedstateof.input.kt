@Composable
fun MyComposable(
    something: State<String> = remember { derivedStateOf { "X" } }
) {
    val something = remember { derivedStateOf { "X" } }
    val something2 by remember { derivedStateOf { "Y" } }
}