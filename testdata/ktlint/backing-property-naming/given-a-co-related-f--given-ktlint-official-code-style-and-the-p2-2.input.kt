class Foo {
    private val _elementList = mutableListOf<Element>()

    protected fun getElementList(): List<Element> = _elementList
}