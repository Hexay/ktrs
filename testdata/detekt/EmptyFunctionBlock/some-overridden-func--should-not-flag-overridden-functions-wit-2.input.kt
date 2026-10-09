private interface Listener {
    fun listenThis()

    fun listenThat()
}

private class AnimationEndListener : Listener {
    override fun listenThis() {
        // no-op
    }

    override fun listenThat() {

    }
}