fun foo(bar: Any): String = when(bar) {
    bar.foobar1(), bar.foobar2() -> "a"
    bar.foobar3(), bar.foobar4(), // The comma should be inserted before the comment
    -> "a"
    bar.foobar5(),
    bar.foobar6(), /* The comma should be inserted before the comment */
    -> "a"
    else -> "b"
}