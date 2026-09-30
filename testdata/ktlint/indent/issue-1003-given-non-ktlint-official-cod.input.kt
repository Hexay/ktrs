abstract class Parent(a: Int, b: Int)

interface Parent2

class Child(
    a: Int,
    b: Int
) : Parent(
    a,
    b
),
    Parent2