val foo1 = if (bar1) {
    "bar1"
} else {
    "bar2"
}.plus("foo")

val foo2 = if (bar1) {
    "bar1"
} else if (bar2) {
    "bar2"
} else {
    "bar3"
}.plus("foo")