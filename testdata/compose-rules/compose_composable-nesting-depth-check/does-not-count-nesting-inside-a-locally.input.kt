@Composable
fun Outer() {
    @Composable
    fun Inner() {
        Box {
            Box {
                Box {
                    Box {
                        Text("hi")
                    }
                }
            }
        }
    }
    Inner()
}