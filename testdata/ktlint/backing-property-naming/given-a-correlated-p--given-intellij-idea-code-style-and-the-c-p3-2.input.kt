class Foo {
    private val _elementList = mutableListOf<Element>()

    internal val elementList: List<Element>
        get() = _elementList
}