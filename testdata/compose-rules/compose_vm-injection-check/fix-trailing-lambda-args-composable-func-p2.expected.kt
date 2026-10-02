@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = weaverViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(text: String, viewModel: MyVM = weaverViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(
    text: String, viewModel: MyVM = weaverViewModel(),
    block: () -> Unit
) {
}