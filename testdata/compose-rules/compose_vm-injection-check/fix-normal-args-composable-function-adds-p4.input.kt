@Composable
fun MyComposable(modifier: Modifier = Modifier) {
    val viewModel: MyVM = bananaViewModel()
}
@Composable
fun MyComposable(modifier: Modifier = Modifier,) {
    val viewModel: MyVM = bananaViewModel()
}