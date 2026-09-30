fun foo() {
    tasks.test {
        systemProperties = mutableMapOf(
            "junit.jupiter.displayname.generator.default" to
                "org.junit.jupiter.api.DisplayNameGenerator\$ReplaceUnderscores",

            "junit.jupiter.execution.parallel.enabled" to
                doParallelTesting.toString() as Any,
            "junit.jupiter.execution.parallel.mode.default" to
                "concurrent",
            "junit.jupiter.execution.parallel.mode.classes.default" to
                "concurrent"
        )
    }
}