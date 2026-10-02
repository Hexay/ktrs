@Composable
fun Content() {
    val viewModel = weaverViewModel<MyVM>()
    key(viewModel) { }
    val x = remember(viewModel) { "ABC" }
    LaunchedEffect(viewModel) { }
}