@Composable
fun TooDeep() {
    Box {
        Box {
            Box {
                Box {
                    Box {
                        Text("")
                    }
                }
            }
        }
    }
}