fun myFunction(a: List<Int>, b: List<Long>, c: List<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: ImmutableList<Int>, b: ImmutableList<Long>, c: ImmutableList<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: PersistentList<Int>, b: PersistentList<Long>, c: PersistentList<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}