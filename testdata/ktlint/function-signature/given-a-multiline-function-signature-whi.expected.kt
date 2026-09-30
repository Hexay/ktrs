// Max line length marker:                    #
// Entire signature is just one character too long to fit on a single line
// .foo1(a: Any, b: Any, c: Any) = "some-result"
fun foo1(a: Any, b: Any, c: Any) =
    "some-result"

// But thanks to max-line-length suppression, the following signature can be written on a single line
@Suppress("ktlint:standard:max-line-length")
fun foo2(a: Any, b: Any, c: Any) = "some-result"