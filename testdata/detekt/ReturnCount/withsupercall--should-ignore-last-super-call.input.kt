open class A {
    open fun a(list: List<Int>) {

    }
}

class B: A() {
    override fun a(list: List<Int>) {
        if (list.isEmpty()) return
        if (list.contains(-1)) return
        if (list.contains(-2)) return
        if (list.contains(-3)) return

        super.a(list)
    }
}