package org.jlleitschuh.gradle.ktlint

import java.nio.charset.StandardCharsets
import java.util.Properties
import org.gradle.api.Action
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.Task
import org.gradle.api.file.ConfigurableFileTree
import org.gradle.api.tasks.util.PatternFilterable
import org.jlleitschuh.gradle.ktlint.tasks.BaseKtLintCheckTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintCheckTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintFormatTask

internal typealias FilterApplier = (Action<PatternFilterable>) -> Unit
internal typealias KotlinScriptAdditionalPathApplier = (ConfigurableFileTree) -> Unit

internal const val KTLINT_PLUGINS_VERSION_PROPERTY = "ktlint-version"
internal const val KTLINT_PLUGINS_PROPERTIES_FILE_NAME = "ktlint-plugins.properties"

/** ktlint-gradle's base plugin: the `ktlint` extension, whose version defaults to `ktlint-plugins.properties`. */
public open class KtlintBasePlugin : Plugin<Project> {
    internal lateinit var extension: KtlintExtension

    override fun apply(target: Project) {
        val filterTargetApplier: FilterApplier = {
            target.tasks.withType(BaseKtLintCheckTask::class.java).configureEach(it)
        }

        val kotlinScriptAdditionalPathApplier: KotlinScriptAdditionalPathApplier = { fileTree ->
            val configureAction =
                Action<Task> { task -> (task as BaseKtLintCheckTask).source(fileTree.also { it.include("*.kts") }) }
            target.tasks.named(KtLintCheckTask.KOTLIN_SCRIPT_TASK_NAME).configure(configureAction)
            target.tasks.named(KtLintFormatTask.KOTLIN_SCRIPT_TASK_NAME).configure(configureAction)
        }

        extension =
            target.extensions.create(
                "ktlint",
                KtlintExtension::class.java,
                target.objects,
                filterTargetApplier,
                kotlinScriptAdditionalPathApplier,
            )
        val propertiesFile = target.layout.projectDirectory.file(KTLINT_PLUGINS_PROPERTIES_FILE_NAME)
        extension.version.set(
            target.providers
                .fileContents(propertiesFile)
                .asText
                .map { text ->
                    val properties = Properties()
                    properties.load(text.byteInputStream(StandardCharsets.UTF_8))
                    properties.getProperty(KTLINT_PLUGINS_VERSION_PROPERTY)?.takeIf { it.isNotBlank() }
                }
                .orElse(KtlintExtension.DEFAULT_KTLINT_VERSION)
        )
    }
}
