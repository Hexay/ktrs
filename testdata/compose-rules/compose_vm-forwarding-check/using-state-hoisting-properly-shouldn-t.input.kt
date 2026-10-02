@Composable
fun MyComposable(viewModel: MyViewModel = weaverViewModel()) {
    val state by viewModel.watchAsState()
    AnotherComposable(state, onAvatarClicked = { viewModel(AvatarClickedIntent) })
}