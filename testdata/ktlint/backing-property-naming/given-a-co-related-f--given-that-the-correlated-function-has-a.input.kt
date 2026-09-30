class Foo {
    fun getElementList(bar: String): List<Element> = _elementList + bar

    private companion object {
        val _elementList = mutableListOf<Element>()
    }
}