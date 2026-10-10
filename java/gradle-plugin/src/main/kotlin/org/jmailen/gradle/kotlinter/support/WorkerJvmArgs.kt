package org.jmailen.gradle.kotlinter.support

internal const val SUN_MISC_UNSAFE_MEMORY_ACCESS_ALLOW = "--sun-misc-unsafe-memory-access=allow"

private const val UNSAFE_WARNING_JVM_VERSION = 24

/** Upstream's default for `workerJvmArgs`, kept for build scripts that read it: ktrs runs no JVM worker. */
internal fun defaultWorkerJvmArgs(jvmMajorVersion: Int = Runtime.version().feature()): List<String> =
    if (jvmMajorVersion >= UNSAFE_WARNING_JVM_VERSION) listOf(SUN_MISC_UNSAFE_MEMORY_ACCESS_ALLOW) else emptyList()
