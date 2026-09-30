fun foo1(i: Int) = -(--i) + 1 + 1
fun foo2(i: Int) = -(++i) + 1 + 1
fun foo3(i: Int) = +1 - 1 + (-1)
fun foo4(): Int {
    var f = 0
    if (-1 < -f && +f > -10) {
        f += -1 + 2 + -3 - 4 + (-4)
    }
    return f
}