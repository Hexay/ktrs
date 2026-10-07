plugins {
    kotlin("jvm") version "2.4.20"
    // the formatter
    id("io.github.hexay.ktrs") version "0.5.0"
}

ktfmt {
    kotlinLangStyle()
}
