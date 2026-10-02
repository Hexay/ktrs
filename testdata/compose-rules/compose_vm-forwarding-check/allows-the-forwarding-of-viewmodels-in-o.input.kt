@Composable
override fun Content() {
    val viewModel = weaverViewModel<MyVM>()
    AnotherComposable(viewModel)
}