@Composable
fun MyComposable(modifier: Modifier,viewModel: MyVM = bananaViewModel()) {
}
@Composable
fun MyComposableNoParams(viewModel: MyVM = bananaViewModel()) {
}
@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = bananaViewModel(), block: () -> Unit) {
}