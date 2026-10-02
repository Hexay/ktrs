@Composable
fun MyComposable(viewModel: MyViewModel) {
    viewModel.let {
        AnotherComposable(it)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    viewModel.let {
        AnotherComposable(vm = it)
    }
}