object MyComposableDefaults
@Composable
fun MyComposable(someParam: Bleh = MyComposableDefaults.someParam) { }
internal object MyOtherComposableDefaults
@Composable
internal fun MyOtherComposable() {
    val someUsage = MyOtherComposableDefaults.someParam.someMethod()
}
object MyThirdComposableDefaults
@Composable
fun MyThirdComposable(a: A) {
    val someUsage = MyThirdComposableDefaults.someParam
}
@Composable
internal fun MyThirdComposable(b: B) {
    val someUsage = MyThirdComposableDefaults.someParam
}