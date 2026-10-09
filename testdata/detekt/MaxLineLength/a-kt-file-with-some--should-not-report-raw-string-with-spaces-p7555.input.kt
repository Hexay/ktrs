class MaxLineLength {
    val longSingleLineRawString1 =
        """This is first yet another very very very very very very very very very very very very very very very very very very very very very very very very very very very very"""

    val longSingleLineRawString2 =
        """
            This is second yet another very very very very very very very very very very very very very very very very very very very very very very very very very very very very"""

    val longSingleLineRawString3 =

        """
            This is third yet another very very very very very very very very very very very very very very very very very very very very very very very very very very very"""

    val longSingleLineRawString4 =

        """

            This is third yet another very very very very very very very very very very very very very very very very very very very very very very very very very very very"""

    val longSingleLineRawString2WithTrimIndent =
        """
            This is second yet another very very very very very very very very very very very very very very very very very very very very very very very very very very very very"""
        .trimIndent()
}