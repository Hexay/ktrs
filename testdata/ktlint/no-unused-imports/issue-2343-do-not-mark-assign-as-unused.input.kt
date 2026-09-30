package com.github.erdi.ktlint.bug

import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.assign
import javax.inject.Inject

class ExamplePlugin @Inject constructor() : Plugin<Project> {
  override fun apply(project: Project) {
    project.let {
      it.javaexec {
        mainClass = "com.github.erdi.ktlint.bug.ExampleMainClass"
      }
    }
  }
}