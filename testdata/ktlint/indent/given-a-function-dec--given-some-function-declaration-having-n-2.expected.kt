fun foo1(bar: String?) =
    bar?.map { it } ?: 0
        ?: -1
fun foo2(bar: String?) =
    bar?.map { it }
        ?: 0
        ?: -1