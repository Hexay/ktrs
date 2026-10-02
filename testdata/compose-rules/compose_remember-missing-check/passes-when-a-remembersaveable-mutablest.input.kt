@Composable
fun MyComposable(
    something: State<String> = rememberSaveable { mutableStateOf("X") }
) {
    val something = rememberSaveable { mutableStateOf("X") }
    val something2 by rememberSaveable { mutableStateOf("Y") }
}