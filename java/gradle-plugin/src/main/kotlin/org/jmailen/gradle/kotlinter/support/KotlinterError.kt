package org.jmailen.gradle.kotlinter.support

import org.gradle.api.GradleException

public class KotlinterError(message: String, error: Throwable) : GradleException(message, error)
