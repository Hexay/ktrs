@Composable
fun MyComposable(viewModel: MyViewComponent) {
    AnotherComposable(viewModel)
}
@Composable
fun MyComposable2(viewModel: MyStateHolder) {
    AnotherComposable(viewModel)
}