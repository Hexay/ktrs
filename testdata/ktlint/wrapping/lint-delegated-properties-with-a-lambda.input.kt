import kotlin.properties.Delegates

class Test {
    private var test
        by Delegates.vetoable("") { _, old, new ->
            true
        }
}