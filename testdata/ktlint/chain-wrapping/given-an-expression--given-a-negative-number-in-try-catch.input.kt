val foo =
    try {
      foo()
      -1
    } catch(e: Exception) {
      -2
    }