@Composable
fun MyComposable(modifier: Modifier) {
    val viewModel = viewModel<MyVM>()
}
@Composable
fun MyComposableNoParams() {
    val viewModel: MyVM = viewModel()
}
@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = viewModel()
}