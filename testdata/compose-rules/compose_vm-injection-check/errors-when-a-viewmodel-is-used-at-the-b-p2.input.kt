@Composable
fun MyComposable(modifier: Modifier) {
    val viewModel = weaverViewModel<MyVM>()
}
@Composable
fun MyComposableNoParams() {
    val viewModel: MyVM = weaverViewModel()
}
@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = weaverViewModel()
}