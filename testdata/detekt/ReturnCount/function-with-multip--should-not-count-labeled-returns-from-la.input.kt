fun readUsers(name: String): Flowable<User> {
return userDao.read(name)
    .flatMap {
        if (it.isEmpty()) return@flatMap Flowable.empty<User>()
        return@flatMap Flowable.just(it[0])
    }
}