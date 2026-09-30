private fun replaceLogger(deviceId: String, orgName: String) {
    val stateManager: StateManager = StateManager()
    stateManager
        .firebaseLogger(
            mode = 0,
            appInstanceIdentity = deviceId,
            org = orgName
        )
    }