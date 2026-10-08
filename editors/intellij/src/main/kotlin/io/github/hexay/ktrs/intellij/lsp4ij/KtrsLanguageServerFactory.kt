package io.github.hexay.ktrs.intellij.lsp4ij

import com.intellij.execution.ExecutionException
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiFile
import com.redhat.devtools.lsp4ij.LanguageServerEnablementSupport
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.LanguageServerManager
import com.redhat.devtools.lsp4ij.client.LanguageClientImpl
import com.redhat.devtools.lsp4ij.client.features.LSPClientFeatures
import com.redhat.devtools.lsp4ij.client.features.LSPFormattingFeature
import com.redhat.devtools.lsp4ij.server.CannotStartProcessException
import com.redhat.devtools.lsp4ij.server.OSProcessStreamConnectionProvider
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import io.github.hexay.ktrs.intellij.KtrsClients
import io.github.hexay.ktrs.intellij.KtrsServer
import io.github.hexay.ktrs.intellij.KtrsSettings
import io.github.hexay.ktrs.intellij.KtrsSettingsListener

/** The server id in `ktrs-lsp4ij.xml`. */
const val SERVER_ID = "ktrs"

@Suppress("UnstableApiUsage")
class KtrsLanguageServerFactory : LanguageServerFactory, LanguageServerEnablementSupport {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider = KtrsConnectionProvider(project)

    override fun createLanguageClient(project: Project): LanguageClientImpl = KtrsLanguageClient(project)

    override fun createClientFeatures(): LSPClientFeatures = LSPClientFeatures().setFormattingFeature(KtrsFormattingFeature())

    override fun isEnabled(project: Project) = KtrsClients.active() == KtrsClients.LSP4IJ

    override fun setEnabled(enabled: Boolean, project: Project) {}
}

@Suppress("UnstableApiUsage")
private class KtrsConnectionProvider(private val project: Project) : OSProcessStreamConnectionProvider() {
    override fun start() {
        commandLine = try {
            KtrsServer.commandLine(project)
        } catch (e: ExecutionException) {
            throw CannotStartProcessException(e.message ?: "no ktrs binary")
        }
        super.start()
    }

    override fun getInitializationOptions(rootUri: VirtualFile?): Any = KtrsServer.settings(project)
}

private class KtrsLanguageClient(project: Project) : LanguageClientImpl(project) {
    override fun createSettings(): Any = KtrsServer.settings(project)
}

private class KtrsFormattingFeature : LSPFormattingFeature() {
    override fun isExistingFormatterOverrideable(file: PsiFile) = KtrsSettings.getInstance(file.project).state.formatsByServer()
}

class KtrsLsp4ijSettingsListener(private val project: Project) : KtrsSettingsListener {
    override fun settingsChanged() {
        val manager = LanguageServerManager.getInstance(project)
        manager.stop(SERVER_ID)
        manager.start(SERVER_ID)
    }
}
