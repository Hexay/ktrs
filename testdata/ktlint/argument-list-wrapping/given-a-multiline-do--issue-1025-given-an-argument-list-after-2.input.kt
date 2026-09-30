private fun replaceLogger(deviceId: String, orgName: String) {
    stateManager
        .firebaseLogger = Logging(
        mode = if (BuildConfig.DEBUG) Logging.Companion.LogDestination.DEV else Logging.Companion.LogDestination.PROD,
        appInstanceIdentity = deviceId,
        org = orgName
    )
    stateManager.firebaseLogger.tellTheCloudAboutMe()
    customisation.attachToFirebase(stateManager.firebaseLogger.appCloudPrefix)
}