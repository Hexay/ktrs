@Composable
fun Something(): String { // This one emits content directly and should fail
    Text("Hi")
    return "Potato"
}
@Composable
fun Something2(): WhateverState { // This one emits content indirectly and should fail too
    Something3()
    return remember { WhateverState() }
}
@Composable
fun Something3() { // This one is fine but calling it should make Something2 fail
    Potato(icon = HorizonIcon.Arrow)
}
@Composable
fun Something4(): String { // This one is using a composable defined in the config
    Banana()
}