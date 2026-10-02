@Composable
fun MyComposable(modifier: Modifier,viewModel: MyVM = potatoViewModel()) {
}
@Composable
fun MyComposableNoParams(viewModel: MyVM = potatoViewModel()) {
}
@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = potatoViewModel(), block: () -> Unit) {
}