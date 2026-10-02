@Composable
fun Something(modifier: Modifier) {
    val tweakedModifier = Modifier.then(modifier).fillMaxWidth()
    val reassignedModifier = modifier
    val modifier3 = Modifier.fillMaxWidth()
    Column(modifier = modifier) {
        OkComposable(modifier = newModifier)
        ComposableThaReusesModifier(modifier = tweakedModifier)
        ComposableThaReusesModifier(modifier = reassignedModifier)
        OkComposable(modifier = modifier3)
    }
    InnerComposable(modifier = tweakedModifier)
}
@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        val tweakedModifier = Modifier.then(modifier).fillMaxWidth()
        val reassignedModifier = modifier
        OkComposable(modifier = newModifier)
        ComposableThaReusesModifier(modifier = tweakedModifier)
        ComposableThaReusesModifier(modifier = reassignedModifier)
    }
}