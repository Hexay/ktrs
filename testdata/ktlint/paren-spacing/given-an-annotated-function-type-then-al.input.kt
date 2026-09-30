val myFun1: @Foo () -> Unit = {}
val myFun2: @Foo() () -> Unit = {}
val typeParameter1: (@Foo () -> Unit) -> Unit = { {} }
val typeParameter2: (@Foo() () -> Unit) -> Unit = { { } }