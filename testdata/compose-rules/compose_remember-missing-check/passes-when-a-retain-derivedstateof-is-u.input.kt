@Composable
fun MyComposable(
    something: State<String> = retain { derivedStateOf { "X" } }
) {
    val something = retain { derivedStateOf { "X" } }
    val something2 by retain { derivedStateOf { "Y" } }
}