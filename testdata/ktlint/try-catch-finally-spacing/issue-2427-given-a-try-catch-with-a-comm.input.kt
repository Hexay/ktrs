fun foo() {
    try {
        trySomething()
    } // Unexpected comment
    catch (e: IOException) {
        catchSomething()
    } // Unexpected comment
    finally {
        finallySomething()
    }
}