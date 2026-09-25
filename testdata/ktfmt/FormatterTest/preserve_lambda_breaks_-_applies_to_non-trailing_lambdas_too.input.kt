fun test() {
  withCallback(
      onClick = {
        log("clicked")
      },
      label = "Press",
  )
}
