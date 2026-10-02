@Composable
fun TooDeep() = Box { Box { Box { Box { Text("x") } } } }
@Composable
fun WithinLimit() = Box { Box { Box { Text("x") } } }