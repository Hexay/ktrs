@Composable
fun MyComposable(modifier: Modifier = Modifier) {
    val viewModel: MyVM = viewModel()
}
@Composable
fun MyComposable(modifier: Modifier = Modifier,) {
    val viewModel: MyVM = viewModel()
}