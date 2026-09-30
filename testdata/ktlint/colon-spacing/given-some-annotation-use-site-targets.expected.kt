@file:JvmName("Foo")
@file:JvmName("Foo")
class Example(@field:Ann val foo: String, @get:Ann val bar: String)
class Example(@field:Ann val foo: String, @get:Ann val bar: String)
class Example {
    @set:[Inject VisibleForTesting]
    public var collaborator: Collaborator
    @set:[Inject VisibleForTesting]
    public var collaborator: Collaborator
}
fun @receiver:Fancy String.myExtension() { }
fun @receiver:Fancy String.myExtension() { }