internal object MyComposableDefaults
@Composable
fun MyComposable(someParam: Bleh = MyComposableDefaults.someParam) { }
private object MyOtherComposableDefaults
@Composable
internal fun MyOtherComposable() {
    val someUsage = MyOtherComposableDefaults.someParam.someMethod()
}