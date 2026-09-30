fun foo(string: String) = string == "foo"
fun main() {
    listOf("foo", "bar", "foobar")
        .map(String::length)
        .map(String::length)
        .map(String::length)
        .map(String::length)
}