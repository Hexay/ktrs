@Composable
fun A(text: String, content: @Composable () -> Unit) {}
@Composable
fun A(content: @Composable Plum) {}
@Composable
fun A(text: String, content: Potato) {}
@Composable
fun A(text: String, content: Banana) {}
@Composable
fun A(text: String, content: Apple) {}

typealias Apple = @Composable () -> Unit

fun interface Banana {
    @Composable fun Content()
}