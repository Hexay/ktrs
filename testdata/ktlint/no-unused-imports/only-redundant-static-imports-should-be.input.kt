import com.foo.psi.abc
import com.foo.psi.findAnnotation

fun main() {
    val psi = getPsi()
    psi.abc()
    val bar = psi.findAnnotation(SOME_CONSTANT)
}