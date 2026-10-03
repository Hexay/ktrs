typealias TypealiasLambda = () -> Unit
fun interface InterfaceLambda {
    fun whatever()
}

fun MyComposable(text1: String, modifier: Modifier = Modifier, other: String = "1", other2: String = "2") { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, other2: String = "2", other : String = "1") { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, trailing: () -> Unit) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: () -> Unit) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: LambdaType) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: LambdaType?) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: (() -> Unit)?) { }

@Composable
fun MyComposable(modifier: Modifier, text1: String, m2: Modifier = Modifier, trailing: (() -> Unit)?) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: TypealiasLambda) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: TypealiasLambda?) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: InterfaceLambda) { }

@Composable
fun MyComposable(text1: String, modifier: Modifier = Modifier, m2: Modifier = Modifier, trailing: InterfaceLambda?) { }