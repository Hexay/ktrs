@Composable
fun WithControlFlow(items: List<String>, cond: Boolean) {
    Box {
        if (cond) {
            for (item in items) {
                Box {
                    Text(item)
                }
            }
        }
    }
}