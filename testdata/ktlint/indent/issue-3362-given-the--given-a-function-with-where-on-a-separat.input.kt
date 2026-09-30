private interface First

private interface Second

private fun <T> example(value: T)
    where T : First,
          T : Second {
    println(value)
}