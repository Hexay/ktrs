// some other content
val x = Regex("""
    Text (.*?)\(in parens\) this is too long to be valid.
    The regex/raw string continues down another line      .
""".trimIndent())
// that is the right length