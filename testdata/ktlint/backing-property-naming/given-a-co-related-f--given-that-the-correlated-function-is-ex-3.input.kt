class Foo {
    public fun getElementList(): List<Element> = _elementList

    companion object {
        private val _elementList = mutableListOf<Element>()
    }
}