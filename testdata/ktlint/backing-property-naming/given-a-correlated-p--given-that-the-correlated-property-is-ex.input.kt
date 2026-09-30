class Foo {
    public val elementList: List<Element>
        get() = _elementList

    private companion object {
        val _elementList = mutableListOf<Element>()
    }
}