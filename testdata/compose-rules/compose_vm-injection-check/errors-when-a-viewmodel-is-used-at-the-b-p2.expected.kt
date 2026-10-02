@Composable
fun MyComposable(modifier: Modifier,viewModel: MyVM = weaverViewModel()) {
}
@Composable
fun MyComposableNoParams(viewModel: MyVM = weaverViewModel()) {
}
@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = weaverViewModel(), block: () -> Unit) {
}