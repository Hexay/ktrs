package io.github.hexay.ktrs.intellij

import com.intellij.openapi.application.PathManager
import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import java.util.Locale

/** Finds the `ktrs` executable: the configured path, else the binary bundled for this platform, else `ktrs` on `PATH`. */
object KtrsBinary {
    const val PLUGIN_ID = "io.github.hexay.ktrs"

    /** The plugin's `bin/<platform>` directories, as `tools/release/bundle-natives.sh` lays them out. */
    val PLATFORMS = listOf("linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64", "windows-x86_64", "windows-aarch64")

    fun resolve(configured: String): Path? =
        resolve(configured, pluginDir(), platform(System.getProperty("os.name"), System.getProperty("os.arch")), System.getenv("PATH"))

    fun resolve(configured: String, pluginDir: Path?, platform: String?, pathEnv: String?): Path? {
        if (configured.isNotBlank()) {
            return Path.of(configured.trim())
        }
        val exe = executableName(platform)
        if (pluginDir != null && platform != null) {
            val bundled = pluginDir.resolve("bin").resolve(platform).resolve(exe)
            if (Files.isRegularFile(bundled)) {
                ensureExecutable(bundled)
                return bundled
            }
        }
        return pathEnv.orEmpty().split(File.pathSeparatorChar).filter { it.isNotBlank() }
            .map { Path.of(it).resolve(exe) }
            .firstOrNull { Files.isRegularFile(it) }
    }

    /** `<os>-<arch>` of a bundled binary, or null when none is built for this OS/arch. */
    fun platform(osName: String, osArch: String): String? {
        val os = osName.lowercase(Locale.ROOT)
        val osPart = when {
            os.startsWith("windows") -> "windows"
            os.startsWith("mac") -> "macos"
            os.startsWith("linux") -> "linux"
            else -> return null
        }
        val archPart = when (osArch.lowercase(Locale.ROOT)) {
            "amd64", "x86_64" -> "x86_64"
            "aarch64", "arm64" -> "aarch64"
            else -> return null
        }
        return "$osPart-$archPart"
    }

    private fun executableName(platform: String?): String {
        val windows = platform?.startsWith("windows") ?: System.getProperty("os.name").startsWith("Windows")
        return if (windows) "ktrs.exe" else "ktrs"
    }

    // <plugin>/lib/<jar>. Not PluginManager(Core) lookups: internal API in Android Studio 2026.2 (verifyPlugin fails).
    private fun pluginDir(): Path? = PathManager.getJarForClass(KtrsBinary::class.java)?.parent?.parent

    // The IDE's plugin installer may drop the zip entries' mode bits.
    private fun ensureExecutable(file: Path) {
        val f = file.toFile()
        if (!f.canExecute()) {
            f.setExecutable(true)
        }
    }
}
