@Composable
fun Something(modifier: Modifier) {
    Column(modifier) {
        InternalComposable()
        Text("Hi")
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier) {
        ComposableWithNewModifier(Modifier.fillMaxWidth())
        Text("Hi", modifier = Modifier.padding12())
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier) {
        val newModifier = Modifier.weight(1f)
        ComposableWithNewModifier(newModifier)
        Text("Hi")
    }
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier) {
        val newModifier = Modifier.weight(1f)
        if(shouldShowSomething) {
            ComposableWithNewModifier(newModifier)
        } else {
            DifferentComposableWithNewModifier(newModifier)
        }
    }
}