@Composable
fun MyComposable() {
    val a = remember { mutableIntListOf() }
    val b = remember { mutableLongListOf() }
    val c = remember { mutableFloatListOf() }
    val d = remember { mutableIntSetOf() }
    val e = remember { mutableLongSetOf() }
    val f = remember { mutableFloatSetOf() }
    val g = remember { mutableIntIntMapOf() }
    val h = remember { mutableLongLongMapOf() }
    val i = remember { mutableFloatFloatMapOf() }
}