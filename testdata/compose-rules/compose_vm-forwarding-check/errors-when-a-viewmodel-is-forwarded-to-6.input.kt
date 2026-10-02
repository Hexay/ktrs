@Composable
fun MyComposable(viewModel: MyViewModel) {
    viewModel.apply {
        AnotherComposable(this)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    viewModel.apply {
        AnotherComposable(vm = this)
    }
}