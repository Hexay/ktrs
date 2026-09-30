val foo1 = if (bar1) {
    "bar1"
} else {
    null
} ?: "something-else"

val foo2 = if (bar1) {
    "bar1"
} else if (bar2) {
    null
} else {
    null
} ?: "something-else"