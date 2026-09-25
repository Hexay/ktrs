fun f() {
  @Suppress("UNCHECKED_CAST") f(1 + f(1) as Int)
  @Suppress("UNCHECKED_CAST") f(1 + f(1) as Int)
}
