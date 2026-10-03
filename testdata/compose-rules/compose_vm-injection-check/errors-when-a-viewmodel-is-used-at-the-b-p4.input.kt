@Composable
fun MyComposable(modifier: Modifier) {
    val viewModel = bananaViewModel<MyVM>()
}
@Composable
fun MyComposableNoParams() {
    val viewModel: MyVM = bananaViewModel()
}
@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = bananaViewModel()
}