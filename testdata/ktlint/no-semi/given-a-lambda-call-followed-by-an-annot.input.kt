annotation class Ann
val f: () -> String = run {
    listOf(1).map {
        it + it
    };
    // Removing the semicolon on the line above results in a compile error!
    @Ann
    { "" }
}