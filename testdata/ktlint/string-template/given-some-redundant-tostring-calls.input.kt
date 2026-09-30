val foo1 = "${String::class.toString()}"
val foo2 = """${Int::class.toString()}"""

class Foo() {
    override fun toString(): String = "Override hashcode = ${super.hashCode().toString()}"
}