@Composable
fun MyComposable(viewModel: MyViewModel) {
    viewModel.also {
        AnotherComposable(it)
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    viewModel.also {
        AnotherComposable(vm = it)
    }
}