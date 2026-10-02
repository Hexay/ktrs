interface MyInterface {
    @Composable
    fun Content() {
        val viewModel = weaverViewModel<MyVM>()
        AnotherComposable(viewModel)
    }
}