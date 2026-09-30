plugins {
    kotlin("jvm") version "2.4.10"
    id("io.github.hexay.ktrs")
}

ktfmt { kotlinLangStyle() }

repositories { mavenCentral() }
