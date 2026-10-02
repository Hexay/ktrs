@Composable
fun MyComposable(modifier: Modifier) {
    val viewModel = hiltViewModel<MyVM>()
}
@Composable
fun MyComposableNoParams() {
    val viewModel: MyVM = hiltViewModel()
}
@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = hiltViewModel()
}