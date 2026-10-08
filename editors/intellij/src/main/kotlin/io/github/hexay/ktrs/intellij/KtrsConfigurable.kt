package io.github.hexay.ktrs.intellij

import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.options.BoundConfigurable
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.DialogPanel
import com.intellij.ui.dsl.builder.AlignX
import com.intellij.ui.dsl.builder.Cell
import com.intellij.ui.dsl.builder.Panel
import com.intellij.ui.dsl.builder.bindItem
import com.intellij.ui.dsl.builder.bindSelected
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel
import com.intellij.ui.dsl.builder.rows
import com.intellij.ui.dsl.builder.textListCellRenderer
import javax.swing.JTextField
import kotlin.reflect.KMutableProperty0

/** Settings | Tools | ktrs. Fields mirror the VS Code extension's settings; "build" = null, the build's value. */
class KtrsConfigurable(private val project: Project) : BoundConfigurable(KtrsServer.NAME) {
    private val state get() = KtrsSettings.getInstance(project).state
    private val local get() = KtrsLocalSettings.getInstance(project).state

    override fun createPanel(): DialogPanel = panel {
        row("ktrs binary:") {
            textFieldWithBrowseButton(FileChooserDescriptorFactory.singleFile().withTitle("ktrs Binary"), project)
                .bindText(local::path).align(AlignX.FILL)
                .comment("Empty: the bundled binary, else <code>ktrs</code> on PATH. Stored per machine.")
        }
        group("Formatting") {
            row("Formatter:") { choice(listOf("auto", "ktfmt", "ktlint", "none"), state::formatTool) }
                .rowComment("auto: the build's ktfmt or ktlint, else none. none keeps the IDE's formatter.")
            row("ktfmt style:") { nullableChoice(listOf("meta", "google", "kotlinlang"), state::ktfmtStyle) }
            row("Max width:") { intField(state::ktfmtMaxWidth) }
            row("Block indent:") { intField(state::ktfmtBlockIndent) }
            row("Continuation indent:") { intField(state::ktfmtContinuationIndent) }
            row("Remove unused imports:") { nullableChoice(listOf(true, false), state::ktfmtRemoveUnusedImports) }
            row("Manage trailing commas:") { nullableChoice(listOf(true, false), state::ktfmtManageTrailingCommas) }
            row { checkBox("Apply .editorconfig (ktfmt --editorconfig)").bindSelected(state::ktfmtEditorconfig) }
        }
        group("ktlint") {
            row("Diagnostics:") { choice(listOf("auto", "true", "false"), state::ktlintEnable) }
                .rowComment("auto: when the build uses ktlint or detects nothing.")
            row("ktlint version:") { nullableChoice(listOf("1.8", "2.0"), state::ktlintVersion) }
            row("Android Studio code style:") { nullableChoice(listOf(true, false), state::ktlintAndroid) }
            row("Experimental rules:") { nullableChoice(listOf(true, false), state::ktlintExperimental) }
            row("EditorConfig overrides:") {
                textArea().rows(3).align(AlignX.FILL).bindText(
                    { state.ktlintEditorconfigOverrides?.entries?.joinToString("\n") { "${it.key} = ${it.value}" }.orEmpty() },
                    { state.ktlintEditorconfigOverrides = parseOverrides(it) },
                ).comment("One <code>name = value</code> per line; win over every .editorconfig. Empty: the build's.")
            }
            row("Rule set JARs:") {
                textArea().rows(2).align(AlignX.FILL).bindText(
                    { state.ktlintRuleSets?.joinToString("\n").orEmpty() },
                    { state.ktlintRuleSets = parseLines(it) },
                ).comment("One path per line; only compose-rules runs. Empty: the build's.")
            }
            row { checkBox("Report violations ktlint can't autocorrect as errors").bindSelected(state::ktlintUnfixableAsError) }
        }
    }

    override fun apply() {
        super.apply()
        project.messageBus.syncPublisher(KtrsSettingsListener.TOPIC).settingsChanged()
    }

    private fun com.intellij.ui.dsl.builder.Row.choice(items: List<String>, prop: KMutableProperty0<String>) =
        comboBox(items).bindItem({ prop.get() }, { prop.set(it ?: items.first()) })

    private fun <T : Any> com.intellij.ui.dsl.builder.Row.nullableChoice(items: List<T>, prop: KMutableProperty0<T?>) =
        comboBox(listOf<T?>(null) + items, textListCellRenderer { it?.toString() ?: "build" }).bindItem(prop)

    private fun com.intellij.ui.dsl.builder.Row.intField(prop: KMutableProperty0<Int?>): Cell<JTextField> =
        textField().columns(6).bindText({ prop.get()?.toString().orEmpty() }, { prop.set(it.trim().toIntOrNull()?.takeIf { n -> n > 0 }) })
            .comment("Empty: the style's.")
}

internal fun parseLines(text: String): MutableList<String>? =
    text.lines().map { it.trim() }.filter { it.isNotEmpty() }.toMutableList().ifEmpty { null }

internal fun parseOverrides(text: String): MutableMap<String, String>? =
    parseLines(text)?.filter { '=' in it }
        ?.associateTo(linkedMapOf()) { it.substringBefore('=').trim() to it.substringAfter('=').trim() }
        ?.ifEmpty { null }
