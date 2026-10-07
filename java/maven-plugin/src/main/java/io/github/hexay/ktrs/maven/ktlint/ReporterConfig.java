package io.github.hexay.ktrs.maven.ktlint;

import java.io.File;
import java.util.Objects;
import java.util.Properties;

/**
 * A {@code <reporter>}: {@code name}, optional {@code output} file and {@code properties}. Equality and hash code are
 * gantsign's Kotlin data class's, which decide the {@code <reporters>} set's order.
 */
public class ReporterConfig {
    private String name;
    private File output;
    private Properties properties;

    public ReporterConfig() {}

    public ReporterConfig(String name) {
        this.name = name;
    }

    public ReporterConfig(String name, File output, Properties properties) {
        this.name = name;
        this.output = output;
        this.properties = properties;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public File getOutput() {
        return output;
    }

    public void setOutput(File output) {
        this.output = output;
    }

    public Properties getProperties() {
        return properties;
    }

    public void setProperties(Properties properties) {
        this.properties = properties;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ReporterConfig)) return false;
        ReporterConfig other = (ReporterConfig) o;
        return Objects.equals(name, other.name) && Objects.equals(output, other.output)
                && Objects.equals(properties, other.properties);
    }

    @Override
    public int hashCode() {
        int result = name == null ? 0 : name.hashCode();
        result = result * 31 + (output == null ? 0 : output.hashCode());
        return result * 31 + (properties == null ? 0 : properties.hashCode());
    }

    @Override
    public String toString() {
        return "ReporterConfig(name=" + name + ", output=" + output + ", properties=" + properties + ")";
    }
}
