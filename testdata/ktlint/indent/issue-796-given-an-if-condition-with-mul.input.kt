private val gpsRegion =
    if (permissionHandler.isPermissionGranted(
            context, Manifest.permission.ACCESS_FINE_LOCATION
        )
    ) {
        // stuff
    }