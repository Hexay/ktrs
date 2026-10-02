fun Modifier.something1(): Modifier = composed {}
fun Modifier.something2() = composed {}
fun Modifier.something3() {
    return composed {}
}