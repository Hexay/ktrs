@Composable
fun A(content: Potato, text: String) {}
@Composable
fun A(content: Apple, text: String) {}
@Composable
fun A(content: Banana, text: String) {}

typealias Apple = @Composable () -> Unit

fun interface Banana {
    @Composable fun Content()
}