@Composable
fun MyComposable(modifier: Modifier = Modifier, other: String, other2: String) { }

@Composable
fun MyComposable(text: String = "deffo", modifier: Modifier = Modifier) { }

@Composable
fun MyComposable(modifier: Modifier = Modifier, text: String = "123", modifier2: Modifier = Modifier) { }

@Composable
fun MyComposable(text: String = "123", modifier: Modifier = Modifier, lambda: () -> Unit) { }

@Composable
fun MyComposable(text1: String, m2: Modifier = Modifier, modifier: Modifier = Modifier, trailing: () -> Unit) { }

@Composable
fun MyComposable(text1: String, m2: Modifier = Modifier, modifier: Modifier = Modifier, trailing: NonFunctionalType) { }

typealias NonFunctionalType = String