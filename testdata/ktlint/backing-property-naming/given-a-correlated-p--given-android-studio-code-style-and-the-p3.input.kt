class Foo {
    internal val elementList: List<Element>
        get() = _elementList

    private companion object {
        val _elementList = mutableListOf<Element>()
    }
}