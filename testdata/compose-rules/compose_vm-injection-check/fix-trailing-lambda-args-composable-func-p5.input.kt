@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = potatoViewModel()
}
@Composable
fun MyComposableTrailingLambda(text: String, block: () -> Unit) {
    val viewModel: MyVM = potatoViewModel()
}
@Composable
fun MyComposableTrailingLambda(
    text: String,
    block: () -> Unit
) {
    val viewModel: MyVM = potatoViewModel()
}