fun main() {
    var x = false && // comment
        false
    x = false &&
        /* comment */
        // comment
        false
    var y = false // comment
        .call()
    y = false
        // comment
        .call()
    y = false // comment
        /* comment */
        .call()
}