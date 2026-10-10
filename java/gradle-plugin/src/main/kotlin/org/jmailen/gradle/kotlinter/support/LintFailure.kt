package org.jmailen.gradle.kotlinter.support

import org.gradle.api.GradleException

public class LintFailure(message: String) : GradleException(message)
