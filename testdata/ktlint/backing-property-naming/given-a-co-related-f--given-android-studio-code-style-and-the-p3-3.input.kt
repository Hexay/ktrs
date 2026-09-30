class Foo {
    internal fun getElementList(): List<Element> = _elementList

    companion object {
        private val _elementList = mutableListOf<Element>()
    }
}