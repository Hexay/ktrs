class Foo {
    private val _elementList = mutableListOf<Element>()

    fun getElementList(bar: String): List<Element> = _elementList + bar
}