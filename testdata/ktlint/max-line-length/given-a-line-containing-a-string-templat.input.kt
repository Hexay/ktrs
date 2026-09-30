// Max line length marker:                                      #
fun foo() {
    throw SomeException(
        "A long exception message followed by a comma-----------",
        e,
    )
    // or
    throw SomeException(
        "A long exception message followed by a (trailing) comma",
    )
}