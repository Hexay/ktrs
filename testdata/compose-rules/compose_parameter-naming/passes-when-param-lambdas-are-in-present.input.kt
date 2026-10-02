@Composable
fun A(
    onClick: () -> Unit,
    onValueChange: (Int) -> Unit,
    onWrite: () -> Unit,
    onPotato: Potato,
    onEmbed: () -> Unit,
    onDone: () -> Unit,
    onFocusChanged: () -> Unit,
    onPlaced: (LayoutCoordinates) -> Unit,
    onValueChangeFinished: () -> Unit,
) {}