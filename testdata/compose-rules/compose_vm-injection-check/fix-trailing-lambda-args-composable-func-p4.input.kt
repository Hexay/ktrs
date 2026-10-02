@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = bananaViewModel()
}
@Composable
fun MyComposableTrailingLambda(text: String, block: () -> Unit) {
    val viewModel: MyVM = bananaViewModel()
}
@Composable
fun MyComposableTrailingLambda(
    text: String,
    block: () -> Unit
) {
    val viewModel: MyVM = bananaViewModel()
}