fun foo(bar: String) = {}
val foo1 = try{foo("bar")}catch (e: Exception){}
val foo2 = {bar: String -> try{foo(bar)}catch (e: Exception){}}