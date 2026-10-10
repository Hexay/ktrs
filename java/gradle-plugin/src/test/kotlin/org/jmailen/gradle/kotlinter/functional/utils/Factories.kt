package org.jmailen.gradle.kotlinter.functional.utils

/** Upstream's tests apply `org.jmailen.kotlinter`. */
internal const val PLUGIN_ID = "io.github.hexay.ktrs.kotlinter"

internal fun kotlinClass(className: String) = "package com.example\n\nobject $className\n"

/** `formatKotlin` fixes the brace spacing and cannot fix the wildcard import. */
internal fun partlyFixableKotlinClass(className: String) =
    """
    import System.*

    class $className{
        private fun hi() {
            out.println("Hello")
        }
    }
    """.trimIndent()

internal val settingsFile = "rootProject.name = 'kotlinter'\n"

internal val androidManifest = "<manifest package=\"com.example\" />\n"

internal val editorConfig = "\n    [*.kt]\n    \n    "

internal val repositories =
    """
    repositories {
        mavenCentral()
    }

    """.trimIndent()

internal val androidRootBuildScript =
    """
    subprojects {
        repositories {
            google()
            mavenCentral()
        }
    }

    """.trimIndent()
