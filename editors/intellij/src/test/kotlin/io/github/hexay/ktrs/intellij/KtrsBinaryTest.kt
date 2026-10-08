package io.github.hexay.ktrs.intellij

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.nio.file.Files
import java.nio.file.Path

class KtrsBinaryTest {
    @get:Rule
    val temp = TemporaryFolder()

    @Test
    fun `platform names match the bundled directories`() {
        assertEquals("linux-x86_64", KtrsBinary.platform("Linux", "amd64"))
        assertEquals("linux-aarch64", KtrsBinary.platform("Linux", "aarch64"))
        assertEquals("macos-x86_64", KtrsBinary.platform("Mac OS X", "x86_64"))
        assertEquals("macos-aarch64", KtrsBinary.platform("Mac OS X", "aarch64"))
        assertEquals("windows-x86_64", KtrsBinary.platform("Windows 11", "amd64"))
        assertEquals("windows-aarch64", KtrsBinary.platform("Windows 11", "arm64"))
        assertNull(KtrsBinary.platform("Linux", "riscv64"))
        assertNull(KtrsBinary.platform("FreeBSD", "amd64"))
    }

    @Test
    fun `configured path wins`() {
        val plugin = pluginWith("linux-x86_64", "ktrs")
        assertEquals(Path.of("/opt/ktrs"), KtrsBinary.resolve(" /opt/ktrs ", plugin, "linux-x86_64", null))
    }

    @Test
    fun `bundled binary for the platform, made executable`() {
        val plugin = pluginWith("linux-x86_64", "ktrs")
        val bundled = plugin.resolve("bin/linux-x86_64/ktrs")
        bundled.toFile().setExecutable(false)
        assertEquals(bundled, KtrsBinary.resolve("", plugin, "linux-x86_64", null))
        if (!System.getProperty("os.name").startsWith("Windows")) {
            assert(Files.isExecutable(bundled))
        }
        val windows = pluginWith("windows-aarch64", "ktrs.exe")
        assertEquals(windows.resolve("bin/windows-aarch64/ktrs.exe"), KtrsBinary.resolve("", windows, "windows-aarch64", null))
    }

    @Test
    fun `falls back to PATH without a bundled binary for the platform`() {
        val plugin = pluginWith("linux-x86_64", "ktrs")
        val empty = temp.newFolder("empty").toPath()
        val bin = temp.newFolder("bin").toPath()
        Files.createFile(bin.resolve("ktrs"))
        val path = listOf(empty, bin).joinToString(File.pathSeparator)
        assertEquals(bin.resolve("ktrs"), KtrsBinary.resolve("", plugin, "linux-aarch64", path))
        assertNull(KtrsBinary.resolve("", plugin, "linux-aarch64", empty.toString()))
    }

    private fun pluginWith(platform: String, exe: String): Path {
        val plugin = temp.newFolder().toPath()
        val dir = Files.createDirectories(plugin.resolve("bin").resolve(platform))
        Files.writeString(dir.resolve(exe), "")
        return plugin
    }
}
