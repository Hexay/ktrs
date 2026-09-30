// Max line length marker:                  #
fun foo() {
    every {
        foo1.bar(bazbazbazbazbazbazbazbazbaz)
    } returns bar
    @Suppress("ktlint:standard:max-line-length")
    every { foo2.bar(bazbazbazbazbazbazbazbazbaz) } returns bar
}