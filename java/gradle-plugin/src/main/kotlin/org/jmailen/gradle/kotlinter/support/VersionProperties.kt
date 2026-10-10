package org.jmailen.gradle.kotlinter.support

import java.util.Properties

public val versionProperties: VersionProperties by lazy { VersionProperties() }

/** `/kotlinter.properties`: the kotlinter release this plugin mirrors and its default ktlint version. */
public class VersionProperties : Properties() {
    init {
        load(this.javaClass.getResourceAsStream("/kotlinter.properties"))
    }

    public fun version(): String = getProperty("version")

    public fun ktlintVersion(): String = getProperty("ktlintVersion")
}
