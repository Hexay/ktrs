@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = viewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(text: String, viewModel: MyVM = viewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(
    text: String, viewModel: MyVM = viewModel(),
    block: () -> Unit
) {
}