@Composable
fun MyComposable() {
    val a = retain { mutableIntListOf() }
    val b = retain { mutableLongListOf() }
    val c = retain { mutableFloatListOf() }
    val d = retain { mutableIntSetOf() }
    val e = retain { mutableLongSetOf() }
    val f = retain { mutableFloatSetOf() }
    val g = retain { mutableIntIntMapOf() }
    val h = retain { mutableLongLongMapOf() }
    val i = retain { mutableFloatFloatMapOf() }
}