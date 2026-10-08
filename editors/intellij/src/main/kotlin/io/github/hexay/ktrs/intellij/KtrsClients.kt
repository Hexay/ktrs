package io.github.hexay.ktrs.intellij

/**
 * Which LSP client runs ktrs: the platform's (`com.intellij.modules.lsp`) when the IDE has it, else LSP4IJ. Both
 * descriptors load when both exist, so each integration asks this before starting a server.
 */
object KtrsClients {
    const val PLATFORM = "platform"
    const val LSP4IJ = "lsp4ij"

    /** Overrides the choice (`platform` or `lsp4ij`); the integration test runs each. */
    const val PROPERTY = "ktrs.lspClient"

    fun active(): String? = System.getProperty(PROPERTY) ?: when {
        loadable("com.intellij.platform.lsp.api.LspServerSupportProvider") -> PLATFORM
        loadable("com.redhat.devtools.lsp4ij.LanguageServerFactory") -> LSP4IJ
        else -> null
    }

    private fun loadable(className: String): Boolean = try {
        Class.forName(className, false, KtrsClients::class.java.classLoader)
        true
    } catch (_: ClassNotFoundException) {
        false
    } catch (_: LinkageError) {
        false
    }
}
