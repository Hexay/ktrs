plugins {
    id("com.ncorti.ktfmt.gradle") version libs.versions.ktfmt.get()
    id("com.diffplug.spotless") version "6.25.0"
}

spotless {
    kotlin {
        ktfmt()
    }
}
