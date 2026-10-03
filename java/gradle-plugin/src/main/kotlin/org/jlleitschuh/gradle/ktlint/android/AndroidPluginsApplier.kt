package org.jlleitschuh.gradle.ktlint.android

import com.android.build.api.dsl.CommonExtension
import java.util.concurrent.Callable
import org.jlleitschuh.gradle.ktlint.KtlintPlugin
import org.jlleitschuh.gradle.ktlint.addSourceSetTasks

private val androidPluginIds =
    listOf("com.android.application", "com.android.library", "com.android.test", "com.android.dynamic-feature")

/**
 * Upstream's `applyKtLintToAndroid`: one check and format task per Android DSL source set (not per
 * variant). Source directories through AGP's public `directories` (upstream reads the internal
 * `DefaultAndroidSourceDirectorySet.srcDirs` of the `kotlin` set, which includes the `java` dirs).
 */
internal fun KtlintPlugin.PluginHolder.applyKtLintToAndroid() {
    androidPluginIds.forEach { id -> target.plugins.withId(id) { configureAndroidSourceSets() } }
}

private fun KtlintPlugin.PluginHolder.configureAndroidSourceSets() {
    val android = target.extensions.findByName("android") as? CommonExtension ?: return
    android.sourceSets.configureEach { sourceSet ->
        // Lazy: source dirs added to the source set later are seen.
        addSourceSetTasks(
            sourceSet.name,
            target.files(Callable { sourceSet.kotlin.directories + sourceSet.java.directories }),
        )
    }
}
