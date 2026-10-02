@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = weaverViewModel()
}
@Composable
fun MyComposableTrailingLambda(text: String, block: () -> Unit) {
    val viewModel: MyVM = weaverViewModel()
}
@Composable
fun MyComposableTrailingLambda(
    text: String,
    block: () -> Unit
) {
    val viewModel: MyVM = weaverViewModel()
}