private val LocalApple = staticCompositionLocalOf<String> { "Apple" }
internal val LocalPlum: ProvidableCompositionLocal<String> = staticCompositionLocalOf { "Plum" }
val LocalPrune = compositionLocalOf { "Prune" }
private val LocalKiwi: ProvidableCompositionLocal<String> = compositionLocalOf { "Kiwi" }
private val LocalLemon = compositionLocalWithComputedDefaultOf { "Lemon" }
private val LocalApricot: ProvidableCompositionLocal<String> = compositionLocalWithComputedDefaultOf { "Apricot" }