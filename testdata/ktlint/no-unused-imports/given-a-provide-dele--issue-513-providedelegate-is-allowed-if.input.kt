import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.tasks.WriteProperties
import org.gradle.kotlin.dsl.getValue
import org.gradle.kotlin.dsl.provideDelegate
import org.gradle.kotlin.dsl.registering

class DumpVersionProperties : Plugin<Project> {
    override fun apply(target: Project) {
        with(target) {
            val dumpVersionProperties by tasks.registering(WriteProperties::class) {
                setProperties(mapOf("version" to "1.2.3"))
                outputFile = rootDir.resolve("version.properties")
            }

        }
    }
}