import foo.`fooBar` // Backticks are redundant but are not disallowed
import foo.bar.`fooBar` // Backticks are redundant but are not disallowed
import foo2.fooBar
import foo2.bar.fooBar

fun main() {
    fooBar()
}