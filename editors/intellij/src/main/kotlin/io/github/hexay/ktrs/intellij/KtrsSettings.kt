package io.github.hexay.ktrs.intellij

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonPrimitive
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.StoragePathMacros
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import com.intellij.util.messages.Topic

/**
 * The server's settings, one field per VS Code `ktrs.*` setting except `path` and `trace.server` (keys, values and
 * meaning: `crates/ktrs-lsp/src/lib.rs`). null leaves the build's value.
 */
data class KtrsState(
    var formatTool: String = "auto",
    var ktfmtStyle: String? = null,
    var ktfmtMaxWidth: Int? = null,
    var ktfmtBlockIndent: Int? = null,
    var ktfmtContinuationIndent: Int? = null,
    var ktfmtRemoveUnusedImports: Boolean? = null,
    var ktfmtManageTrailingCommas: Boolean? = null,
    var ktfmtEditorconfig: Boolean = false,
    var ktlintEnable: String = "auto",
    var ktlintVersion: String? = null,
    var ktlintAndroid: Boolean? = null,
    var ktlintExperimental: Boolean? = null,
    var ktlintEditorconfigOverrides: MutableMap<String, String>? = null,
    var ktlintRuleSets: MutableList<String>? = null,
    var ktlintUnfixableAsError: Boolean = false,
) {
    /** The `ktrs` section as VS Code sends it: nested objects, nulls kept. */
    fun toJson(): JsonObject = obj(
        "format" to obj("tool" to json(formatTool)),
        "ktfmt" to obj(
            "style" to json(ktfmtStyle),
            "maxWidth" to json(ktfmtMaxWidth),
            "blockIndent" to json(ktfmtBlockIndent),
            "continuationIndent" to json(ktfmtContinuationIndent),
            "removeUnusedImports" to json(ktfmtRemoveUnusedImports),
            "manageTrailingCommas" to json(ktfmtManageTrailingCommas),
            "editorconfig" to json(ktfmtEditorconfig),
        ),
        "ktlint" to obj(
            "enable" to json(ktlintEnable),
            "version" to json(ktlintVersion),
            "android" to json(ktlintAndroid),
            "experimental" to json(ktlintExperimental),
            "editorconfigOverrides" to (ktlintEditorconfigOverrides?.let { m -> obj(*m.map { (k, v) -> k to json(v) }.toTypedArray()) } ?: JsonNull.INSTANCE),
            "ruleSets" to (ktlintRuleSets?.let { l -> JsonArray().apply { l.forEach { add(it) } } } ?: JsonNull.INSTANCE),
            "unfixableAsError" to json(ktlintUnfixableAsError),
        ),
    )

    /** Reformat Code goes to the server, not the IDE's Kotlin formatter. */
    fun formatsByServer(): Boolean = formatTool != "none"

    /** `initializationOptions` and `workspace/didChangeConfiguration`'s settings. */
    fun toServerSettings(): JsonObject = obj("ktrs" to toJson())

    private fun obj(vararg entries: Pair<String, JsonElement>) = JsonObject().apply { entries.forEach { (k, v) -> add(k, v) } }

    private fun json(value: Any?): JsonElement = when (value) {
        null -> JsonNull.INSTANCE
        is String -> JsonPrimitive(value)
        is Number -> JsonPrimitive(value)
        is Boolean -> JsonPrimitive(value)
        else -> error("unsupported ${value::class}")
    }
}

/** Shared with the project (`.idea/ktrs.xml`). */
@Service(Service.Level.PROJECT)
@State(name = "KtrsSettings", storages = [Storage("ktrs.xml")])
class KtrsSettings : PersistentStateComponent<KtrsState> {
    private var state = KtrsState()

    override fun getState(): KtrsState = state

    override fun loadState(state: KtrsState) {
        this.state = state
    }

    companion object {
        fun getInstance(project: Project): KtrsSettings = project.service()
    }
}

/** The `ktrs` path, per machine: `.idea/workspace.xml`, so a cloned project can't pick the executable. */
@Service(Service.Level.PROJECT)
@State(name = "KtrsLocalSettings", storages = [Storage(StoragePathMacros.WORKSPACE_FILE)])
class KtrsLocalSettings : PersistentStateComponent<KtrsLocalSettings.LocalState> {
    data class LocalState(var path: String = "")

    private var state = LocalState()

    override fun getState(): LocalState = state

    override fun loadState(state: LocalState) {
        this.state = state
    }

    companion object {
        fun getInstance(project: Project): KtrsLocalSettings = project.service()
    }
}

/** Published after the settings change; each LSP integration restarts its server. */
fun interface KtrsSettingsListener {
    fun settingsChanged()

    companion object {
        @Topic.ProjectLevel
        val TOPIC = Topic(KtrsSettingsListener::class.java, Topic.BroadcastDirection.NONE)
    }
}
