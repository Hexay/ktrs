@file:Suppress("UnstableApiUsage", "DEPRECATION")

package io.github.hexay.ktrs.intellij.lsp

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.intellij.platform.lsp.api.customization.LspCustomization
import com.intellij.platform.lsp.api.customization.LspFormattingCustomizer
import com.intellij.platform.lsp.api.customization.LspFormattingSupport
import io.github.hexay.ktrs.intellij.KtrsClients
import io.github.hexay.ktrs.intellij.KtrsServer
import io.github.hexay.ktrs.intellij.KtrsSettings
import io.github.hexay.ktrs.intellij.KtrsSettingsListener

// The pre-2026.1.4 names (LspServerSupportProvider, ...): the renamed ones don't exist in 253.
class KtrsLspServerSupportProvider : LspServerSupportProvider {
    override fun fileOpened(project: Project, file: VirtualFile, serverStarter: LspServerSupportProvider.LspServerStarter) {
        if (KtrsServer.isKotlinFile(file) && KtrsClients.active() == KtrsClients.PLATFORM) {
            serverStarter.ensureServerStarted(KtrsLspServerDescriptor(project))
        }
    }
}

private class KtrsLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, KtrsServer.NAME) {
    override fun isSupportedFile(file: VirtualFile) = KtrsServer.isKotlinFile(file)

    override fun getLanguageId(file: VirtualFile) = "kotlin"

    override fun createCommandLine() = KtrsServer.commandLine(project)

    override fun createInitializationOptions(): Any = KtrsServer.settings(project)

    override val lspCustomization = object : LspCustomization() {
        override val formattingCustomizer: LspFormattingCustomizer = object : LspFormattingSupport() {
            override fun shouldFormatThisFileExclusivelyByServer(
                file: VirtualFile,
                ideCanFormatThisFileItself: Boolean,
                serverExplicitlyWantsToFormatThisFile: Boolean,
            ) = KtrsServer.isKotlinFile(file) && KtrsSettings.getInstance(project).state.formatsByServer()
        }
    }
}

class KtrsLspSettingsListener(private val project: Project) : KtrsSettingsListener {
    override fun settingsChanged() {
        LspServerManager.getInstance(project).stopAndRestartIfNeeded(KtrsLspServerSupportProvider::class.java)
    }
}
