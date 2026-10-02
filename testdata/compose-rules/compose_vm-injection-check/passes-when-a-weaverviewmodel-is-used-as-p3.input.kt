@Composable
fun MyComposable(
    modifier: Modifier,
    viewModel: MyVM = hiltViewModel(),
    viewModel2: MyVM = hiltViewModel(),
) { }