fun foo() {
    if (
      -3 == foo()
    ) {}
    if (
      // comment
      -3 == foo()
    ) {}
    if (
      /* comment */
      -3 == foo()
    ) {}
}