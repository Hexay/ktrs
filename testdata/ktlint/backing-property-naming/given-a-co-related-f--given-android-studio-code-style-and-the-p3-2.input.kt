class Foo {
    private val _elementList = mutableListOf<Element>()

    internal fun getElementList(): List<Element> = _elementList
}