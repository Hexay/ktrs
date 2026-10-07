plugins {
    id("com.diffplug.spotless")
}

spotless {
    kotlin {
        ktfmt("0.64").googleStyle()
    }
}
