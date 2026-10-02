@Composable
fun A() {
    when {
        isPotato -> {
            Text("1")
            Text("2")
        }
        else -> {
            Text("1")
            Text("2")
        }
    }
}
@Composable
fun B() {
    when {
        isPotato -> {
            Text("1")
        }
        else -> {
            Text("1")
            Text("2")
        }
    }
}
@Composable
fun C() {
    when {
        isPotato -> {
            Text("1")
            Text("2")
        }
        else -> {
            Text("1")
        }
    }
}
@Composable
fun D() {
    Text("1")
    when {
        isPotato -> Text("2")
        else -> {}
    }
}