plugins {
    `kotlin-dsl`
}

gradlePlugin {
    plugins {
        register("mySpotless") {
            id = libs.plugins.my.spotless.get().pluginId
            implementationClass = "SpotlessConventionPlugin"
        }
    }
}
