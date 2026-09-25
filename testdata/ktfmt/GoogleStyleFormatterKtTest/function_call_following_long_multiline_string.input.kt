fun f() {
  val str1 =
    """
    Some very long string that might mess things up
    """
      .trimIndent()

  val str2 =
    """
    Some very long string that might mess things up
    """
      .trimIndent(someArg)
}
