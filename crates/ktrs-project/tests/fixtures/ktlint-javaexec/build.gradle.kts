val ktlint by configurations.creating

dependencies {
    ktlint("com.pinterest.ktlint:ktlint-cli:1.8.0") {
        attributes {
            attribute(Bundling.BUNDLING_ATTRIBUTE, objects.named(Bundling.EXTERNAL))
        }
    }
    ktlint(project(":rules"))
}

val ktlintCheck by tasks.registering(JavaExec::class) {
    classpath = ktlint
    mainClass.set("com.pinterest.ktlint.Main")
    args("src/**/*.kt")
}
