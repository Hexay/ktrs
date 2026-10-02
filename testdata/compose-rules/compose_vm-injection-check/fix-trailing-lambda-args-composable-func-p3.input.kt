@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = hiltViewModel()
}
@Composable
fun MyComposableTrailingLambda(text: String, block: () -> Unit) {
    val viewModel: MyVM = hiltViewModel()
}
@Composable
fun MyComposableTrailingLambda(
    text: String,
    block: () -> Unit
) {
    val viewModel: MyVM = hiltViewModel()
}