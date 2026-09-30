package io.github.hexay.ktrs.gradle

import com.ncorti.ktfmt.gradle.KtfmtExtension
import com.ncorti.ktfmt.gradle.tasks.KtfmtBaseTask
import com.ncorti.ktfmt.gradle.util.i
import io.github.hexay.ktrs.gradle.KtfmtAndroidUtils.applyKtfmtToAndroidProject
import io.github.hexay.ktrs.gradle.KtfmtPluginUtils.EXTENSION_NAME
import io.github.hexay.ktrs.gradle.KtfmtPluginUtils.TASK_NAME_CHECK
import io.github.hexay.ktrs.gradle.KtfmtPluginUtils.TASK_NAME_FORMAT
import io.github.hexay.ktrs.gradle.KtfmtPluginUtils.createScriptsTasks
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.Task
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.TaskProvider

/**
 * `io.github.hexay.ktrs`: ktfmt-gradle 0.27.0's `KtfmtPlugin` (same extension, tasks and wiring),
 * formatting through [KtrsBuildService] instead of ktfmt in Gradle workers.
 */
public abstract class KtrsPlugin : Plugin<Project> {

    override fun apply(project: Project) {
        val ktfmtExtension = project.extensions.create(EXTENSION_NAME, KtfmtExtension::class.java)
        val service = registerService(project)
        val version = ktrsVersion()

        project.tasks.withType(KtfmtBaseTask::class.java).configureEach {
            it.formattingOptionsBean.set(ktfmtExtension.toFormattingOptions())
            it.useClassloaderIsolation.set(ktfmtExtension.useClassloaderIsolation)
            it.ktrsVersion.set(version)
            it.ktrsService.set(service)
            it.usesService(service)
        }

        val topLevelFormat = createTopLevelFormatTask(project)
        val topLevelCheck = createTopLevelCheckTask(project)

        createScriptsTasks(project, project.projectDir, topLevelFormat, topLevelCheck)

        val kotlin = { KotlinSourceSets(project, ktfmtExtension, topLevelFormat, topLevelCheck) }
        project.plugins.withId("kotlin") { kotlin().applyKtfmt() }

        val applyAndroidKtfmt = {
            if (project.plugins.hasPlugin("org.jetbrains.kotlin.multiplatform")) {
                project.logger.i("Skipping Android task creation, as KMP is applied")
            } else {
                applyKtfmtToAndroidProject(project, topLevelFormat, topLevelCheck, ktfmtExtension)
            }
        }
        project.plugins.withId("com.android.application") { applyAndroidKtfmt() }
        project.plugins.withId("com.android.library") { applyAndroidKtfmt() }
        project.plugins.withId("com.android.test") { applyAndroidKtfmt() }
        project.plugins.withId("com.android.dynamic-feature") { applyAndroidKtfmt() }

        project.plugins.withId("org.jetbrains.kotlin.js") { kotlin().applyKtfmt() }
        project.plugins.withId("org.jetbrains.kotlin.multiplatform") {
            kotlin().applyKtfmtToMultiplatformProject()
        }
    }

    private fun registerService(project: Project): Provider<KtrsBuildService> {
        val property = KtrsBuildService.EXECUTABLE_PROPERTY
        return project.gradle.sharedServices.registerIfAbsent(
            KtrsBuildService.NAME,
            KtrsBuildService::class.java,
        ) {
            it.parameters.executable.set(
                project.providers
                    .gradleProperty(property)
                    .orElse(project.providers.systemProperty(property))
            )
        }
    }

    private fun ktrsVersion(): String =
        KtrsPlugin::class.java.getResource("ktrs-version.txt")?.readText()
            ?: error("Missing ktrs version")

    private fun createTopLevelFormatTask(project: Project): TaskProvider<Task> {
        return project.tasks.register(TASK_NAME_FORMAT) {
            it.group = "formatting"
            it.description = "Run Ktfmt formatter for all source sets for project '${project.name}'"
        }
    }

    private fun createTopLevelCheckTask(project: Project): TaskProvider<Task> {
        return project.tasks.register(TASK_NAME_CHECK) {
            it.group = "verification"
            it.description =
                "Run Ktfmt validation for all source sets for project '${project.name}'"
        }
    }
}
