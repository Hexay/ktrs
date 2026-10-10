package org.jmailen.gradle.kotlinter.tasks.format

import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.Property
import org.gradle.workers.WorkParameters

/** Upstream's parameters, then what the `ktrs ktlint` run needs. */
public interface FormatWorkerParameters : WorkParameters {
    public val name: Property<String>
    public val changedEditorConfigFiles: ConfigurableFileCollection
    public val files: ConfigurableFileCollection
    public val projectDirectory: RegularFileProperty
    public val output: RegularFileProperty

    public val ktlintVersion: Property<String>
    public val ktlintClasspath: ConfigurableFileCollection
    public val ktrsExecutable: Property<String>
    public val temporaryDir: DirectoryProperty
}
