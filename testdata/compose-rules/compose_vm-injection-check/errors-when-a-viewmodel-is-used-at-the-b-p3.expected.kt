@Composable
fun MyComposable(modifier: Modifier,viewModel: MyVM = hiltViewModel()) {
}
@Composable
fun MyComposableNoParams(viewModel: MyVM = hiltViewModel()) {
}
@Composable
fun MyComposableTrailingLambda(viewModel: MyVM = hiltViewModel(), block: () -> Unit) {
}