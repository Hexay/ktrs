@Composable
fun MyComposable(modifier: Modifier,viewModel: MyVM = viewModel()) {
}
@Composable
fun MyComposableNoParams(viewModel: MyVM = viewModel()) {
}
@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = viewModel(), block: () -> Unit) {
}