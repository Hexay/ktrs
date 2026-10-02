val AppleLocal = staticCompositionLocalOf<String> { "Apple" }
val Plum: ProvidableCompositionLocal<String> = staticCompositionLocalOf { "Plum" }
private val Lemon = compositionLocalWithComputedDefaultOf { "Lemon" }