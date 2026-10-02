@Composable
fun MyComposable(
    something: State<String> = retain { mutableStateOf("X") }
) {
    val something = retain { mutableStateOf("X") }
    val something2 by retain { mutableStateOf("Y") }
}