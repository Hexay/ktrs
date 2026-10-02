@Composable
fun MyComposable(modifier: Modifier) {
    if (blah) {
        val viewModel = viewModel<MyVM>()
    } else {
        val viewModel: MyOtherVM = viewModel()
    }
}