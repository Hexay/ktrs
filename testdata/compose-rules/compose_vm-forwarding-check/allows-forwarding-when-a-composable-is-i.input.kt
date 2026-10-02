@Composable
fun MyComposable(viewModel: MyViewModel) {
    AnotherComposableContent(viewModel)
}
@Composable
fun MyComposable2(viewModel: MyViewModel) {
    Row {
        AnotherComposableContent(viewModel)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    AnotherComposableContent(vm = viewModel)
}