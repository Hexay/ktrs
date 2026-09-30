fun foo1() {
    foo2(
        """${
            true
        }
    text
_${
            true
        }
        """.trimIndent(),
        """text"""
    )
}