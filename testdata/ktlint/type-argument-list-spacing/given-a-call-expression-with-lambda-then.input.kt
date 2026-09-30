val foo = compareBy<Foo> { foo -> foo.x() }
    .thenBy { 99 }