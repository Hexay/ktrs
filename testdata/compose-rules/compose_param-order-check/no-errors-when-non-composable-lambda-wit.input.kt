@Composable
fun Foo(
  onSelect: (Foo) -> Unit,
  displayMapper: (Foo) -> String = { it.name },
  title: String,
) { }

@Composable
fun Bar(
  onClick: () -> Unit,
  formatter: (String) -> String = { it },
  text: String,
  count: Int,
) { }

@Composable
fun Baz(
  modifier: Modifier = Modifier,
  onValueChange: (Int) -> Unit = {},
  value: Int,
) { }