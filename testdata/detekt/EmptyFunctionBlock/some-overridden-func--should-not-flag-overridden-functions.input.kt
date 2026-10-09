fun empty() {}

open class Base {
    open fun stuff() {}
}

class A : Base() {
    override fun stuff() {}
}

class B : Base() {
    override fun stuff() {
        TODO("Implement this")
    }
}

class C : Base() {
    override fun stuff() {
        // this is necessary...
    }
}