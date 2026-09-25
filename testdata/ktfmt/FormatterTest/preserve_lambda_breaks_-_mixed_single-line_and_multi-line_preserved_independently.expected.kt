fun compose() {
  App {
    val state = remember { mutableStateOf(0) }
    SelectableCard {
      Button { Text("Count: ${state.value}") }
    }
  }
}
