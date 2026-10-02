@Composable
fun MyComposable(viewModel: MyViewModel) {
    AnotherComposable(viewModel)
}
@Composable
fun MyComposable2(viewModel: MyViewModel) {
    Row {
        AnotherComposable(viewModel)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    AnotherComposable(vm = viewModel)
}