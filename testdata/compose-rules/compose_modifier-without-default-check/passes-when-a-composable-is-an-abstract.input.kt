abstract class Bleh {
    @Composable
    abstract fun Something(modifier: Modifier)

    @Composable
    open fun Something(modifier: Modifier) {}
}