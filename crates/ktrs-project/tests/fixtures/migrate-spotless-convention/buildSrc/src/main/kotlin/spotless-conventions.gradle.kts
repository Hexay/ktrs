plugins {
    id("com.diffplug.spotless")
}

spotless {
    kotlin {
        ktfmt("0.65").googleStyle()
    }
}
