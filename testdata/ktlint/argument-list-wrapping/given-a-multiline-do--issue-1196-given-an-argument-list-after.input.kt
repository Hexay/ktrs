class MyClass {
    private fun initCoilOkHttp() {
        Coil.setImageLoader(
            ImageLoader.Builder(this)
                .crossfade(true)
                .okHttpClient(
                    okHttpClient.newBuilder()
                        .cache(CoilUtils.createDefaultCache(this))
                        .build()
                )
                .build()
        )
    }
}