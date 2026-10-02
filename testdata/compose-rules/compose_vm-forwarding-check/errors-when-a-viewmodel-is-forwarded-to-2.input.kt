@Composable
fun MyComposable(viewModel: MyViewModel) {
    viewModel.run {
        AnotherComposable(this)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    viewModel.run {
        AnotherComposable(vm = this)
    }
}