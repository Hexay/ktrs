fun foo(string: String) = string == "foo"
val predicateA: (String) -> Boolean = ::foo
val predicateB: (String) -> Boolean = ::foo
val predicateC: (String) -> Boolean = ::foo
val predicateD: (String) -> Boolean = ::foo
val predicateE: (String) -> Boolean =
    ::foo
val predicateF: (String) -> Boolean = ::foo