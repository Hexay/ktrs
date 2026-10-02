@Composable
fun MyComposable(viewModel: MyViewModel) {
    with(viewModel) {
        AnotherComposable(this)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    with(viewModel) {
        AnotherComposable(vm = this)
    }
}