var x = 1
fun multipleGuards(a: Int?, b: Any?, c: Int?, d: Int?) {
    if(a == null) return
    val models = b as? Int ?: return
    val position = c?.takeIf { it != -1 } ?: return
    x = d ?: return
    if(b !is String) {
        println("b is not a String")
        return
    }

    return
}