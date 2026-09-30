class Foo {
    private val _elementList = mutableListOf<Element>()

    private val elementList: List<Element>
        get() = _elementList
}