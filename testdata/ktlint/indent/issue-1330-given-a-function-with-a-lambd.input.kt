fun func(lambdaArg: Unit.() -> Unit = {}, secondArg: Int) {
    println()
}
fun func(lambdaArg: Unit.(a: String) -> Unit = { it -> it.toUpperCaseAsciiOnly() }, secondArg: Int) {
    println()
}