@Composable
fun Something(onClick: () -> Unit) {
    LaunchedEffect(Unit) {
        viewModel.onClick()
    }
}