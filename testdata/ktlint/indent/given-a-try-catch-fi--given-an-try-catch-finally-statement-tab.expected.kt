fun foo() =
	try {
		"foo1"
	} catch (e: Exception) {
		"foo2"
	} finally {
		Unit // do something
	}