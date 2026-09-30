class Bar {
    // Backing property
    private val _elementList = mutableListOf<Element>()
    val elementList: List<Element>
        get() = _elementList

    // Backing property defined in companion object
    val elementList2: List<Element>
        get() = _elementList2

    companion object {
        private val _elementList2 = mutableListOf<Element>()
    }
}