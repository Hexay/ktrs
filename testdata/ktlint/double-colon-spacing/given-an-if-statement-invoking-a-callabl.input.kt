fun foo(string: String) = string == "foo"
fun main() {
    if (true == ::foo.invoke("")) {
        // do stuff
    }
}