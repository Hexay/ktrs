@Composable
fun MyComposable(viewModel: PotatoViewModel) {
    AnotherComposable(viewModel)
}
@Composable
fun MyComposable2(viewModel: PotatoViewModel) {
    Row {
        AnotherComposable(viewModel)
    }
}
@Composable
fun MyComposable3(viewModel: PotatoViewModel) {
    AnotherComposable(vm = viewModel)
}
@Composable
fun MyComposable4(viewModel: PotatoViewModel) {
    with(viewModel) { AnotherComposable(vm = this) }
}