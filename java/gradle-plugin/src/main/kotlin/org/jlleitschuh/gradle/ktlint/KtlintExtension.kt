package org.jlleitschuh.gradle.ktlint

import javax.inject.Inject
import org.gradle.api.Action
import org.gradle.api.NamedDomainObjectContainer
import org.gradle.api.file.ConfigurableFileTree
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.MapProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.util.PatternFilterable
import org.jlleitschuh.gradle.ktlint.reporter.CustomReporter
import org.jlleitschuh.gradle.ktlint.reporter.ReporterType

/**
 * ktlint-gradle 14.2.0's `ktlint { }` extension. `version` takes "1.8.0" (the default) or
 * "2.0.0-ALPHA-4"; `android` and `enableExperimentalRules` are accepted and, as upstream with
 * ktlint 1.x, have no effect (use `ktlint_code_style` / `ktlint_experimental` in `.editorconfig`).
 *
 * @param filterTargetApplier When [KtlintExtension.filter] is called, this function is executed.
 */
public open class KtlintExtension
@Inject
internal constructor(
    objectFactory: ObjectFactory,
    projectLayout: ProjectLayout,
    private val filterTargetApplier: FilterApplier,
    kotlinScriptAdditionalPathApplier: KotlinScriptAdditionalPathApplier,
) {
    public val reporterExtension: ReporterExtension =
        objectFactory.newInstance(ReporterExtension::class.java)

    /** The version of KtLint to use: "1.8.0" or "2.0.0-ALPHA-4". */
    public val version: Property<String> =
        objectFactory.property(String::class.java).value(DEFAULT_KTLINT_VERSION)

    /** Enable relative paths in reports. */
    public val relative: Property<Boolean> = objectFactory.property(Boolean::class.java).value(false)

    /** Enable verbose mode: the rule id after each console error. */
    public val verbose: Property<Boolean> = objectFactory.property(Boolean::class.java).value(false)

    /** Enable debug mode: the ktrs command lines and output at warn level. */
    public val debug: Property<Boolean> = objectFactory.property(Boolean::class.java).value(false)

    /** Enable android mode. No effect, as upstream with ktlint 1.x. */
    public val android: Property<Boolean> = objectFactory.property(Boolean::class.java).value(false)

    /** Enable console output mode. */
    public val outputToConsole: Property<Boolean> =
        objectFactory.property(Boolean::class.java).value(true)

    /** Enable colored output in reports. */
    public val coloredOutput: Property<Boolean> =
        objectFactory.property(Boolean::class.java).value(true)

    /** Specify the color of the terminal output. */
    public val outputColorName: Property<String> =
        objectFactory.property(String::class.java).value("")

    /** Whether to allow the build to continue if there are warnings. */
    public val ignoreFailures: Property<Boolean> =
        objectFactory.property(Boolean::class.java).value(false)

    /**
     * Configure Ktlint output reporters. _By default_ `plain` reporter is enabled, if no other
     * reporter is configured.
     */
    public fun reporters(action: Action<ReporterExtension>) {
        action.execute(reporterExtension)
    }

    /** Enable experimental ktlint rules. No effect, as upstream with ktlint 1.x. */
    public val enableExperimentalRules: Property<Boolean> =
        objectFactory.property(Boolean::class.java).value(false)

    /** Additional `.editorconfig` properties, overriding the project's `.editorconfig` files. */
    public val additionalEditorconfig: MapProperty<String, String> =
        objectFactory.mapProperty(String::class.java, String::class.java).apply {
            convention(emptyMap())
        }

    /** Baseline file location. Default is `<projectDir>/config/ktlint/baseline.xml`. */
    public val baseline: RegularFileProperty =
        objectFactory
            .fileProperty()
            .convention(projectLayout.projectDirectory.dir("config").dir("ktlint").file("baseline.xml"))

    private val kscriptExtension = KScriptExtension(kotlinScriptAdditionalPathApplier)

    /** Additional, relative to the project base dir, paths that contain kotlin script files. */
    public fun kotlinScriptAdditionalPaths(action: Action<KScriptExtension>) {
        action.execute(kscriptExtension)
    }

    /** Filter sources by applying exclude or include specs/patterns. */
    public fun filter(filterAction: Action<PatternFilterable>) {
        filterTargetApplier(filterAction)
    }

    public class KScriptExtension(private val ksApplier: KotlinScriptAdditionalPathApplier) {
        /** Adds given [fileTree] to kotlin script check/format tasks search. */
        public fun include(fileTree: ConfigurableFileTree) {
            ksApplier(fileTree)
        }
    }

    public open class ReporterExtension @Inject constructor(objectFactory: ObjectFactory) {
        internal val reporters: SetProperty<ReporterType> =
            objectFactory.setProperty(ReporterType::class.java).value(emptySet())

        public val customReporters: NamedDomainObjectContainer<CustomReporter> =
            objectFactory.domainObjectContainer(CustomReporter::class.java) { CustomReporter(it) }

        /** Use one of the ktlint reporters: `plain`, `plain_group_by_file`, `checkstyle`, `json`, `sarif`, `html`. */
        public fun reporter(reporterType: ReporterType) {
            reporters.add(reporterType)
        }

        /** Add 3rd party reporters. */
        public fun customReporters(configuration: Action<NamedDomainObjectContainer<CustomReporter>>) {
            configuration.execute(customReporters)
        }
    }

    internal companion object {
        const val DEFAULT_KTLINT_VERSION = "1.8.0"
    }
}
