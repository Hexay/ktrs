class MyCliktCommand : CliktCommand() {
    private val myOption
        by {
            option("--myOption")
                .int()
                .default(1)
        }
}