@Composable
fun Something(modifier: Modifier) {
    Column(modifier = modifier) {
        TrustedFriendsMembersAppBar(
            onBackClicked = { viewModel.processUserIntent(BackClicked) },
            onDoneClicked = { viewModel.processUserIntent(DoneClicked) }
        )

        val recommendedEmptyUsersContent: @Composable ((Modifier) -> Unit)? = when {
            !recommended.isEmpty -> null
            searchQuery.value.isEmpty() -> { localModifier: Modifier ->
                EmptyUsersList(
                    title = stringResource(trustedR.string.trusted_friends_members_list_empty_title),
                    description = stringResource(trustedR.string.trusted_friends_members_list_empty_description),
                    modifier = localModifier
                )
            }
            else -> { localModifier ->
                EmptyUsersList(
                    title = stringResource(trustedR.string.trusted_friends_search_empty_title),
                    description = stringResource(
                        trustedR.string.trusted_friends_search_empty_description,
                        searchQuery.value
                    ),
                    modifier = localModifier
                )
            }
        }
    }
}