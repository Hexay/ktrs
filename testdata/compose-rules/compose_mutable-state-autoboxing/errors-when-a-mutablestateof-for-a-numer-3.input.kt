fun myFunction(a: Set<Int>, b: Set<Long>, c: Set<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: ImmutableSet<Int>, b: ImmutableSet<Long>, c: ImmutableSet<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}
fun myFunction(a: PersistentSet<Int>, b: PersistentSet<Long>, c: PersistentSet<Float>) {
    var a by mutableStateOf(a)
    var b by mutableStateOf(b)
    var c by mutableStateOf(c)
}