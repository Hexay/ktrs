import io.github.hexay.ktrs.KtlintOptions
import io.github.hexay.ktrs.KtrsOptions
import io.github.hexay.ktrs.spotless.KtrsKtlintStep

plugins {
    id("com.diffplug.spotless")
}

spotless {
    kotlin {
        addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(KtrsOptions.kotlinlang().withMaxWidth(120)))
        addStep(
            KtrsKtlintStep.create(
                KtlintOptions.of("2.0.0-ALPHA-4")
                    .withEditorConfigOverride(mapOf("indent_size" to 2))
                    .withCustomRuleSets(listOf(file("rules/compose.jar"))),
            ),
        )
    }
}
