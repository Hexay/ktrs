@Composable
fun MyComposable(modifier: Modifier = Modifier) {
    val viewModel: MyVM = weaverViewModel()
}
@Composable
fun MyComposable(modifier: Modifier = Modifier,) {
    val viewModel: MyVM = weaverViewModel()
}