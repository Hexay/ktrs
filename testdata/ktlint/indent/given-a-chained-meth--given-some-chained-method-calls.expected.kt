fun foo1(bar: String) =
    bar.uppercase(Locale.getDefault())
        .trim()
        .length.also {
            println("done")
        }