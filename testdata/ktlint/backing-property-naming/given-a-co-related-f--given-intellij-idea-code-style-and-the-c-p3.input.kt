class Foo {
    internal fun getElementList(): List<Element> = _elementList

    private companion object {
        val _elementList = mutableListOf<Element>()
    }
}