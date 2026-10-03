@Composable
fun Something(modifier: Modifier = Modifier) {
    if (someCondition) {
        Case1RootLevelComposable(modifier = modifier.background(HorizonColor.Black))
    } else {
        Case2RootLevelComposable(modifier)
    }
}