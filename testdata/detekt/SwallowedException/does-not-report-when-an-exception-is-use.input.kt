fun Exception.transformException(): Exception {
   return this
}

fun test() {
   try {
   } catch (e: Exception) {
      throw e.transformException()
   }
}