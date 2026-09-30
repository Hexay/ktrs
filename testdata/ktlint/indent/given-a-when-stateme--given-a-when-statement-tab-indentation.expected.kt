fun foo(bar: Any) =
	when (bar) {
		is Number -> 0
		else -> 1
	}