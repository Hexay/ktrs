fun test(i: Int, j: Int): Int {
    return if (i == 1) {
        if (j == 1) {
            1
        } else {
            2
        }
    } else if (i == 2) {
        if (j == 1) {
            3
        } else {
            4
        }
    } else {
        if (j == 1) {
            5
        } else {
            6
        }
    }
}