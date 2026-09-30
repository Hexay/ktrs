class Foo {
    fun getElementList(bar: String): List<Element> = _elementList + bar

    companion object {
        private val _elementList = mutableListOf<Element>()
    }
}