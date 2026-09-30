class A {
    fun f(@Annotation
        a: Any,
        @Annotation([
            "v1",
            "v2"
        ])
        b: Any,
        c: Any =
            false,
        @Annotation d: Any,
        @SingleLineAnnotation([1, 2])
        e: Any) {
    }
}