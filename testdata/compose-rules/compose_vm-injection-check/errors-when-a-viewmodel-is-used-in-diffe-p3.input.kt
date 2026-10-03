@Composable
fun MyComposable(modifier: Modifier) {
    if (blah) {
        val viewModel = hiltViewModel<MyVM>()
    } else {
        val viewModel: MyOtherVM = hiltViewModel()
    }
}