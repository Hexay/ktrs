@Composable
fun MyComposable(modifier: Modifier) {
    if (blah) {
        val viewModel = weaverViewModel<MyVM>()
    } else {
        val viewModel: MyOtherVM = weaverViewModel()
    }
}