val LocalBanana = staticCompositionLocalOf<String> { "Banana" }
val LocalPotato = compositionLocalOf { "Potato" }
val LocalPeach = compositionLocalWithComputedDefaultOf { "Peach" }