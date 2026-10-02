@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = hiltViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(text: String, viewModel: MyVM = hiltViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(
    text: String, viewModel: MyVM = hiltViewModel(),
    block: () -> Unit
) {
}