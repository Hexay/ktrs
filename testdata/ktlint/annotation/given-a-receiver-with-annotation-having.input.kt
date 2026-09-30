annotation class Ann(val arg: Int = 0)

fun @receiver:Ann(1) String.test() {}
