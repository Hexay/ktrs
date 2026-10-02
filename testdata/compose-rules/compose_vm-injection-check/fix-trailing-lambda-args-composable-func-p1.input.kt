@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = viewModel()
}
@Composable
fun MyComposableTrailingLambda(text: String, block: () -> Unit) {
    val viewModel: MyVM = viewModel()
}
@Composable
fun MyComposableTrailingLambda(
    text: String,
    block: () -> Unit
) {
    val viewModel: MyVM = viewModel()
}