class Foo {
    private val _elementList = mutableListOf<Element>()

    private fun getElementList(): List<Element> = _elementList
}