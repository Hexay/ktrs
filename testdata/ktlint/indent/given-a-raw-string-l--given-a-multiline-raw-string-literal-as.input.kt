fun foo() {
println("""
    text

        text
""".trimIndent().toByteArray())
}