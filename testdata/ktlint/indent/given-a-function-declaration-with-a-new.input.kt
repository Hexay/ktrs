private fun getImplementationVersion() =
    javaClass.`package`.implementationVersion
        ?: javaClass.getResourceAsStream("/META-INF/MANIFEST.MF")
            ?.let { stream ->
                Manifest(stream).mainAttributes.getValue("Implementation-Version")
            }