package io.github.hexay.ktrs.gradle

import com.ncorti.ktfmt.gradle.KtfmtExtension
import io.github.hexay.ktrs.gradle.KtfmtAndroidUtils.applyKtfmtToAndroidProject
import io.github.hexay.ktrs.gradle.KtfmtPluginUtils.createTasksForSourceSet
import org.gradle.api.Project
import org.gradle.api.Task
import org.gradle.api.tasks.TaskProvider
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.jetbrains.kotlin.gradle.dsl.KotlinProjectExtension
import org.jetbrains.kotlin.gradle.plugin.KotlinPlatformType

/**
 * ktfmt-gradle's `KtfmtPlugin.applyKtfmt*`, kept out of [KtrsPlugin] so the plugin class loads
 * without the Kotlin Gradle plugin on its classpath.
 */
internal class KotlinSourceSets(
    private val project: Project,
    private val ktfmtExtension: KtfmtExtension,
    private val topLevelFormat: TaskProvider<Task>,
    private val topLevelCheck: TaskProvider<Task>,
) {

    fun applyKtfmt() {
        val extension = project.extensions.getByType(KotlinProjectExtension::class.java)
        extension.sourceSets.configureEach {
            createTasksForSourceSet(
                project,
                it.name,
                it.kotlin.sourceDirectories,
                ktfmtExtension,
                topLevelFormat,
                topLevelCheck,
            )
        }
    }

    fun applyKtfmtToMultiplatformProject() {
        val extension = project.extensions.getByType(KotlinMultiplatformExtension::class.java)

        // This plugin's Android target is a plain KMP target (`KotlinPlatformType.jvm`, not
        // `androidJvm`), so its "android*" source sets take the regular path.
        val hasAndroidKmpLibraryPlugin =
            project.plugins.hasPlugin("com.android.kotlin.multiplatform.library")

        extension.sourceSets.configureEach {
            val name = "kmp ${it.name}"
            if (!hasAndroidKmpLibraryPlugin && it.name.startsWith("android")) {
                // Created from the Android DSL below instead.
                return@configureEach
            }
            createTasksForSourceSet(
                project,
                name,
                it.kotlin.sourceDirectories,
                ktfmtExtension,
                topLevelFormat,
                topLevelCheck,
            )
        }

        extension.targets.configureEach { kotlinTarget ->
            if (kotlinTarget.platformType == KotlinPlatformType.androidJvm) {
                applyKtfmtToAndroidProject(
                    project,
                    topLevelFormat,
                    topLevelCheck,
                    ktfmtExtension,
                    isKmpProject = true,
                )
            }
        }
    }
}
