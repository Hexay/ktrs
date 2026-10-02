@Composable
fun MyComposable(modifier: Modifier) {
    NavHost() {
        composable("bleh") {
            val viewModel = hiltViewModel<MyVM>()
        }
    }
}