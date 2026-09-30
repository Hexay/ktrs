var setterVisibility: String = "abc"
    private set
var setterWithAnnotation: Any? = null
    @Inject set
var setterOnNextLine: String
    private set(value) { setterOnNextLine = value}