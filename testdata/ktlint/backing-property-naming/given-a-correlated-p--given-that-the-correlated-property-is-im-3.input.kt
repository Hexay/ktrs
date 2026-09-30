class Foo {
    val elementList: List<Element>
        get() = _elementList

    companion object {
        private val _elementList = mutableListOf<Element>()
    }
}