dependencies {
    ktlintRuleset(libs.compose.rules)
}

ktlint {
    enableExperimentalRules.set(true)
    additionalEditorconfig.put("ktlint_code_style", "intellij_idea")
}
