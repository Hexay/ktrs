val foo = if (System.currentTimeMillis() % 2 == 0L) {
    0
} else System.currentTimeMillis().toInt()