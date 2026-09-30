fun foo() {
    if (
        listOf<Any>()
            .toString()
            .isEmpty() &&
        false ||
        (
            true ||
                false ||
                listOf<Any>()
                    .toString()
                    .isEmpty()
            ) ||
        false
    ) {
        println("hello")
    }
}