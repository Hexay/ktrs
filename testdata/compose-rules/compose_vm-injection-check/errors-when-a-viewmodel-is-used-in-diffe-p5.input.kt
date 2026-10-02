@Composable
fun MyComposable(modifier: Modifier) {
    if (blah) {
        val viewModel = potatoViewModel<MyVM>()
    } else {
        val viewModel: MyOtherVM = potatoViewModel()
    }
}