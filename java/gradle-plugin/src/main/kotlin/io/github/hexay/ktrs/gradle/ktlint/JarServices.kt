package io.github.hexay.ktrs.gradle.ktlint

import java.io.ByteArrayOutputStream
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipFile
import java.util.zip.ZipOutputStream
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.component.ModuleComponentIdentifier
import org.gradle.api.file.FileCollection

/**
 * Turns resolved `ktlintRuleset` / `ktlintReporter` files into ktlint CLI `-R` / `artifact=` JARs.
 * Upstream puts the whole configuration on a worker classpath; the CLI loads each JAR on its own
 * and rejects one that declares no provider, so a rule set with dependencies (compose-rules'
 * Maven artifact needs `common-ktlint`) becomes one merged JAR.
 */
internal object JarServices {

    // What the ktlint jar itself provides: resolving them again would shadow its own classes.
    private val providedGroups =
        listOf("com.pinterest", "io.github.ktlint", "org.jetbrains", "io.github.oshai", "org.slf4j", "org.ec4j", "ch.qos.logback")

    /** [configuration]'s files without ktlint and its runtime (Kotlin, logging, ec4j). */
    fun withoutKtlintItself(configuration: Configuration): FileCollection =
        configuration.incoming
            .artifactView { view ->
                view.componentFilter { id ->
                    !(id is ModuleComponentIdentifier && providedGroups.any { id.group == it || id.group.startsWith("$it.") })
                }
            }
            .files

    /**
     * The `-R` JARs for [files] and ktlint [version]: none when no JAR declares a rule set provider,
     * the JAR itself when it is the only one (so ktrs recognizes a release it runs natively), else
     * all of them merged into one JAR in [tempDir].
     */
    fun ruleSetJars(files: Iterable<File>, version: String, tempDir: File): List<File> {
        val jars = files.filter { it.isFile && it.name.endsWith(".jar") }
        val interfaces = KtlintVersions.ruleSetInterfaces(version)
        return when {
            jars.none { jar -> interfaces.any { declaresService(jar, it) } } -> emptyList()
            jars.size == 1 -> jars
            else -> listOf(merge(jars, File(tempDir, "ktlint-rulesets.jar")))
        }
    }

    /** The reporter JARs among [files] for ktlint [version]. */
    fun reporterJars(files: Iterable<File>, version: String): List<File> =
        files.filter { it.isFile && declaresService(it, KtlintVersions.reporterInterface(version)) }

    private fun declaresService(jar: File, serviceInterface: String): Boolean =
        try {
            ZipFile(jar).use { it.getEntry("$SERVICES$serviceInterface") != null }
        } catch (_: java.io.IOException) {
            false
        }

    /** [jars]' entries in one JAR: the first of duplicate entries wins, service files are concatenated. */
    private fun merge(jars: List<File>, target: File): File {
        target.parentFile.mkdirs()
        val services = linkedMapOf<String, ByteArrayOutputStream>()
        val written = mutableSetOf<String>()
        ZipOutputStream(target.outputStream().buffered()).use { out ->
            for (jar in jars) {
                ZipFile(jar).use { zip ->
                    for (entry in zip.entries()) {
                        val name = entry.name
                        when {
                            entry.isDirectory || isSignature(name) -> {}
                            name.startsWith(SERVICES) ->
                                services.getOrPut(name) { ByteArrayOutputStream() }.apply {
                                    zip.getInputStream(entry).use { it.copyTo(this) }
                                    write('\n'.code)
                                }
                            written.add(name) -> {
                                out.putNextEntry(ZipEntry(name))
                                zip.getInputStream(entry).use { it.copyTo(out) }
                                out.closeEntry()
                            }
                        }
                    }
                }
            }
            services.forEach { (name, content) ->
                out.putNextEntry(ZipEntry(name))
                out.write(content.toByteArray())
                out.closeEntry()
            }
        }
        return target
    }

    private fun isSignature(name: String): Boolean =
        name.startsWith("META-INF/") && listOf(".SF", ".DSA", ".RSA", ".EC").any { name.endsWith(it) }

    private const val SERVICES = "META-INF/services/"
}
