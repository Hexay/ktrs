@file:Suppress("ktlint:compose:defaults-visibility")

internal object MyComposableDefaults
@Composable
fun MyComposable(someParam: Bleh = MyComposableDefaults.someParam) { }