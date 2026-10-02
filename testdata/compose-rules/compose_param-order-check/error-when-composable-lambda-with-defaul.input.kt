@Composable
fun Foo(
  onClick: () -> Unit,
  content: @Composable () -> Unit = {},
  title: String,
) { }