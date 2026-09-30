class Bar1 {
    // Incomplete backing property as public property 'elementList' or function `getElementList` is missing
    private val _elementList = mutableListOf<Element>()
}
class Bar2 {
    // Invalid backing property as '_elementList' is not a private property
    val _elementList = mutableListOf<Element>()
    val elementList: List<Element>
        get() = _elementList2
}
class Bar3 {
    // Invalid backing property as 'elementList' is not a public property
    // Note: code below is allowed in `android_studio` code style!
    private val _elementList = mutableListOf<Element>()
    internal val elementList: List<Element>
        get() = _elementList2
}