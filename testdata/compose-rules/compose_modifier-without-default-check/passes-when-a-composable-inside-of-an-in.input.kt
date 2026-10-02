interface Bleh {
    @Composable
    fun Something(modifier: Modifier)
}
class BlehImpl : Bleh {
    @Composable
    override fun Something(modifier: Modifier) {}
}
@Composable
actual fun Something(modifier: Modifier) {}