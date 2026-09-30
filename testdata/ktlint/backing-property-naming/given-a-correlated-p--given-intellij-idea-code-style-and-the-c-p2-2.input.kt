class Foo {
    private val _elementList = mutableListOf<Element>()

    protected val elementList: List<Element>
        get() = _elementList
}