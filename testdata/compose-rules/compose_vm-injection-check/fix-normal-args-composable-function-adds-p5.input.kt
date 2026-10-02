@Composable
fun MyComposable(modifier: Modifier = Modifier) {
    val viewModel: MyVM = potatoViewModel()
}
@Composable
fun MyComposable(modifier: Modifier = Modifier,) {
    val viewModel: MyVM = potatoViewModel()
}