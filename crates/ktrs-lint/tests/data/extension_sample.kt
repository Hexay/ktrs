@file:Suppress("ktlint:standard:max-line-length")

package a.b.c

import x.y.Z
import x.y.* // all of it

/**
 * KDoc with a [link] and a @tag.
 */
@Target(AnnotationTarget.CLASS)
annotation class Marker

private val s = "é𝄞 ${1 + 2} $s"

class Foo<T>(val a: Int, private var b: String = "x") : Bar(), Baz by q {
    init {
        println(a) ; println(b)
    }

    @Suppress("ktlint:standard:max-line-length")
    fun long(): String = """
        |line one
        |line ${two()}
    """.trimMargin()

    fun whenever(x: Any) = when (x) {
        is Int -> 1 // one
        in 1..2, 3 -> 2
        else -> { /* block */ 3 }
    }

	fun tabbed() {
		val l = listOf(1, 2).map { it * 2 }.filter { v -> v > 1 }
	}


    companion object {
        const val K = 1; val J = 2
    }
}

fun <R> R.ext(block: R.() -> Unit): R where R : Any = apply(block)

val lambda = @Suppress("x") { y: Int ->
    y + 1
}
