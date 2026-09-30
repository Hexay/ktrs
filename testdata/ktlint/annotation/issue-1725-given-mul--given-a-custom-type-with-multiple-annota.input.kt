val fooBar1: FooBar<String, @Foo String, @Foo @Bar String, @Bar("bar") @Foo String> = FooBar()
val fooBar2: FooBar<String, @Foo String, @Foo @Bar String, @Baz("baz") @Foo String> = FooBar()