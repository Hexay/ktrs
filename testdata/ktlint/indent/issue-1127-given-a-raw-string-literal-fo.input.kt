interface UserRepository : JpaRepository<User, UUID> {
    @Query(
        """
        select u from User u
        inner join Organization o on u.organization = o
        where o = :organization
    """.trimIndent()
    )
    fun findByOrganization(organization: Organization, pageable: Pageable): Page<User>
}