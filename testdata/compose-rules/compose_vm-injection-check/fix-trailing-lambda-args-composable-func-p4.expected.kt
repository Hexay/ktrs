@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = bananaViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(text: String, viewModel: MyVM = bananaViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(
    text: String, viewModel: MyVM = bananaViewModel(),
    block: () -> Unit
) {
}