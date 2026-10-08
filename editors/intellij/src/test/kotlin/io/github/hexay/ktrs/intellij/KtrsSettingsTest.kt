package io.github.hexay.ktrs.intellij

import com.google.gson.JsonObject
import com.google.gson.JsonParser
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.nio.file.Files
import java.nio.file.Path

class KtrsSettingsTest {
    /** The VS Code extension's `ktrs.*` settings, minus the client-only ones, nested by key with their defaults. */
    private fun vscodeDefaults(): JsonObject {
        val packageJson = JsonParser.parseString(Files.readString(Path.of("../vscode/package.json"))).asJsonObject
        val properties = packageJson.getAsJsonObject("contributes").getAsJsonObject("configuration").getAsJsonObject("properties")
        val section = JsonObject()
        for ((key, schema) in properties.entrySet()) {
            if (key == "ktrs.path" || key == "ktrs.trace.server") continue
            val parts = key.removePrefix("ktrs.").split('.')
            var target = section
            for (part in parts.dropLast(1)) {
                target = target.getAsJsonObject(part) ?: JsonObject().also { target.add(part, it) }
            }
            target.add(parts.last(), schema.asJsonObject.get("default"))
        }
        return section
    }

    @Test
    fun `defaults have the VS Code extension's keys and values`() {
        assertEquals(vscodeDefaults(), KtrsState().toJson())
    }

    @Test
    fun `values map to the server's nested keys under ktrs`() {
        val state = KtrsState(
            formatTool = "ktfmt", ktfmtStyle = "google", ktfmtMaxWidth = 120, ktfmtRemoveUnusedImports = false,
            ktfmtEditorconfig = true, ktlintEnable = "false", ktlintVersion = "2.0", ktlintAndroid = true,
            ktlintEditorconfigOverrides = linkedMapOf("max_line_length" to "100"), ktlintRuleSets = mutableListOf("/r.jar"),
            ktlintUnfixableAsError = true,
        )
        val expected = JsonParser.parseString(
            """{"ktrs": {"format": {"tool": "ktfmt"},
              "ktfmt": {"style": "google", "maxWidth": 120, "blockIndent": null, "continuationIndent": null,
                "removeUnusedImports": false, "manageTrailingCommas": null, "editorconfig": true},
              "ktlint": {"enable": "false", "version": "2.0", "android": true, "experimental": null,
                "editorconfigOverrides": {"max_line_length": "100"}, "ruleSets": ["/r.jar"], "unfixableAsError": true}}}""",
        )
        assertEquals(expected, state.toServerSettings())
    }

    @Test
    fun `text areas parse to null when empty`() {
        assertNull(parseLines(" \n "))
        assertEquals(listOf("a.jar", "b.jar"), parseLines("a.jar\n\n b.jar "))
        assertNull(parseOverrides("no equals sign"))
        assertEquals(mapOf("ktlint_standard_no-wildcard-imports" to "disabled", "indent_size" to "2"),
            parseOverrides("ktlint_standard_no-wildcard-imports = disabled\nindent_size=2"))
    }
}
