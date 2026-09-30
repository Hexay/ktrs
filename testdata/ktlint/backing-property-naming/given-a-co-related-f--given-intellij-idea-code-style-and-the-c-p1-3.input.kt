class Foo {
    private fun getElementList(): List<Element> = _elementList

    companion object {
        private val _elementList = mutableListOf<Element>()
    }
}