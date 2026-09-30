class Foo {
    private val _elementList = mutableListOf<Element>()

    public val elementList: List<Element>
        get() = _elementList
}