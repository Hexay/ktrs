fun foo() {
    write(fs.getPath("/projects/.editorconfig"), """
        root = true
        [*]
        end_of_line = lf
    """.trimIndent().toByteArray())
}