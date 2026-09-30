fun foo() {
    every { foo() }returns(bar)andThen(baz)
    every { foo() }returns (bar)andThen (baz)
    every { foo() } returns(bar) andThen(baz)
}