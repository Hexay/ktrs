@Composable
fun MyComposable(modifier: Modifier) {
    val viewModel = potatoViewModel<MyVM>()
}
@Composable
fun MyComposableNoParams() {
    val viewModel: MyVM = potatoViewModel()
}
@Composable
fun MyComposableTrailingLambda(block: () -> Unit) {
    val viewModel: MyVM = potatoViewModel()
}