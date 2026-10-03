package org.jlleitschuh.gradle.ktlint

import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.jetbrains.kotlin.gradle.dsl.KotlinProjectExtension

/**
 * Upstream `KtlintPlugin.applyKtLint*`, kept out of [KtlintPlugin] so the plugin class loads
 * without the Kotlin Gradle plugin on its classpath.
 */
internal class KotlinSourceSetsApplier(private val holder: KtlintPlugin.PluginHolder) {

    fun applyKtLint() {
        holder.target.extensions.configure(KotlinProjectExtension::class.java) { extension ->
            extension.sourceSets.all { holder.addSourceSetTasks(it.name, it.kotlin.sourceDirectories) }
        }
    }

    fun applyKtlintMultiplatform() {
        val extension = holder.target.extensions.getByType(KotlinMultiplatformExtension::class.java)
        // Upstream's `targets.all { if (androidJvm) applyKtLintToAndroid() }` only builds a lambda and
        // drops it, so a KMP Android target gets no tasks beyond its Kotlin source sets' here either.
        extension.sourceSets.all { holder.addSourceSetTasks(it.name, it.kotlin.sourceDirectories) }
    }
}
