interface TaskContainer {
    fun register(name: String, block: Number.() -> Unit = {})
}
interface Project {
    val tasks: TaskContainer
}
fun repros(project: Project) {
    val part = "name".capitalize()
    project.tasks.register("shortName${part}WithSuffix")
    project.tasks.register("veryVeryVeryVeryVeryVeryLongName${part}WithSuffix1")
    project.tasks.register("veryVeryVeryVeryVeryVeryLongName${part}WithSuffix2") {
        this.toByte()
    }
    project.tasks
        .register("veryVeryVeryVeryVeryVeryLongName${part}WithSuffix3") {
        this.toByte()
    }
}