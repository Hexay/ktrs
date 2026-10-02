@Composable
fun Something(modifier: Modifier = Modifier) {
    Row(modifier = modifier) {
        Text("Hi")
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    Potato(modifier.fillMaxWidth()) {
        Text("Hi")
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    val poop = if (x) modifier else modifier.fillMaxWidth()
    Column(modifier = poop) {
        Text("Hi")
    }
}
@Composable
fun Something(modifier: Modifier = Modifier) {
    if (paella.isWellDone()) {
        Column(modifier) {
            Text("Yay")
        }
    } else {
        Row(modifier) {
            Text("Oh no")
        }
    }
}
@Composable
fun Something(
  modifier: Modifier = Modifier,
  content: @Composable BoxScope.() -> Unit
) {
  MaterialTheme(
    colorScheme = darkColorScheme()
  ) {
    Box(
      modifier = modifier
        .fillMaxSize()
        .background(
          color = MaterialTheme.colorScheme.background
        )
    ) {
      Card(
        modifier = Modifier.fillMaxSize()
      ) {
        Box(
          modifier = Modifier.padding(16.dp)
        ) {
          content()
        }
      }
    }
  }
}