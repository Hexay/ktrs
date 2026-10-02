fun myFunction(a: Map<Int, Int>, b: Map<Int, Long>, c: Map<Int, Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: Map<Long, Int>, b: Map<Long, Long>, c: Map<Long, Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: Map<Float, Int>, b: Map<Float, Long>, c: Map<Float, Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}