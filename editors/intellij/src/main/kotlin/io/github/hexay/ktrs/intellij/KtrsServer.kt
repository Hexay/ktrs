package io.github.hexay.ktrs.intellij

import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import java.nio.file.Files
import java.nio.file.Path

/** What both LSP integrations share: which files ktrs serves and how it is launched. */
object KtrsServer {
    const val NAME = "ktrs"

    fun isKotlinFile(file: VirtualFile): Boolean = file.extension == "kt" || file.extension == "kts"

    fun commandLine(project: Project): GeneralCommandLine {
        val configured = KtrsLocalSettings.getInstance(project).state.path
        val binary = KtrsBinary.resolve(configured) ?: throw ExecutionException(
            "No ktrs binary for this platform: install ktrs (https://github.com/Hexay/ktrs#installation) " +
                "or set its path in Settings | Tools | ktrs.",
        )
        return GeneralCommandLine(binary.toString(), "lsp")
            // basePath may not exist (default project, tests); a missing working directory fails the launch.
            .withWorkingDirectory(project.basePath?.let { Path.of(it) }?.takeIf { Files.isDirectory(it) })
            .withCharset(Charsets.UTF_8)
    }

    fun settings(project: Project) = KtrsSettings.getInstance(project).state.toServerSettings()
}
