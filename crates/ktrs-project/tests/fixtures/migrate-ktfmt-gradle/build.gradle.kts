plugins {
    kotlin("jvm") version "2.4.20"
    // the formatter
    id("com.ncorti.ktfmt.gradle") version "0.27.0"
}

ktfmt {
    kotlinLangStyle()
}
