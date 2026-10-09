class C : I {
    override fun someStuff(`object`: String) {}
}
interface I { fun someStuff(@Suppress("FunctionParameterNaming") `object`: String) }