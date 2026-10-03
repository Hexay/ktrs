@Composable
fun MyComposable(modifier: Modifier = Modifier) {
    val viewModel: MyVM = hiltViewModel()
}
@Composable
fun MyComposable(modifier: Modifier = Modifier,) {
    val viewModel: MyVM = hiltViewModel()
}