@Composable
fun MyComposable(
    something: State<String> = remember { mutableStateOf("X") }
) {
    val something = remember { mutableStateOf("X") }
    val something2 by remember { mutableStateOf("Y") }
}