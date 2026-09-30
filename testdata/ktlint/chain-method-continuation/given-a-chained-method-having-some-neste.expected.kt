// Max line length marker:                        #
fun buildBar1(): Foo.Bar = Foo
    .baz()
    .Bar
    .builder()
    .build()
fun buildBar2(): Foo.Bar = Foo
    .baz()
    .bar.Bar
    .builder()
    .build()
fun buildBar3(): Foo.Bar = Foo
    .baz()
    .bar.bar.Bar
    .builder()
    .build()