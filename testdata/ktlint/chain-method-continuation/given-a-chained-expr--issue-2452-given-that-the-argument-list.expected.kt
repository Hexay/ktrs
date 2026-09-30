val foo1 = requireNotNull(bar
    .filter { it == 'b' }
    .filter { it == 'a' }
    .filter { it == 'r' })
val foo2 = requireNotNull(bar.filter { it == 'b' }.filter { it == 'a' }.filter { it == 'r' })