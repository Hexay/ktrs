class Foo {
    protected fun getElementList(): List<Element> = _elementList

    private companion object {
        val _elementList = mutableListOf<Element>()
    }
}