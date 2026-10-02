var a by mutableStateOf("")
var b by mutableStateOf(true)
var c by remember { mutableStateOf(0U) }
var d by mutableStateOf(0u)
var e by mutableStateOf(0UL)
var f by mutableStateOf(0uL)
fun bleh(c: String) {
    var ccc by mutableStateOf(c)
}