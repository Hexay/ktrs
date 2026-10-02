@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = potatoViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(text: String, viewModel: MyVM = potatoViewModel(), block: () -> Unit) {
}
@Composable
fun MyComposableTrailingLambda(
    text: String, viewModel: MyVM = potatoViewModel(),
    block: () -> Unit
) {
}