// Assume that Bar is defined outside the scope of the project. As of that the function signature can not be changed to comply
// to the function-naming rule. In case that Bar is defined inside the scope of the project, the violation will still be
// reported in the Bar class/interface itself.
class Foo : Bar {
    override fun Something()
}