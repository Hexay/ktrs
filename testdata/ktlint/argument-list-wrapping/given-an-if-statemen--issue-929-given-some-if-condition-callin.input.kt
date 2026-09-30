fun test(param1: Int, param2: Int) {
    if (listOfNotNull(
            param1
        ).isEmpty()
    ) {
        println(1)
    } else if (listOfNotNull(
            param2
        ).isEmpty()
    ) {
        println(2)
    }
}