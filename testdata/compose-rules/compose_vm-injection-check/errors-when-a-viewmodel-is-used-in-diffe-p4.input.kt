@Composable
fun MyComposable(modifier: Modifier) {
    if (blah) {
        val viewModel = bananaViewModel<MyVM>()
    } else {
        val viewModel: MyOtherVM = bananaViewModel()
    }
}