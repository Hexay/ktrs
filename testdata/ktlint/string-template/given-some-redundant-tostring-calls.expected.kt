val foo1 = "${String::class}"
val foo2 = """${Int::class}"""

class Foo() {
    override fun toString(): String = "Override hashcode = ${super.hashCode()}"
}