open class A {
    open fun a(list: List<Int>) {

    }
}

class B: A() {
    override fun a(list: List<Int>) {
        super.a(list)

        if (list.isEmpty()) return
        if (list.contains(-1)) return
        if (list.contains(-2)) return
        if (list.contains(-3)) return
    }
}