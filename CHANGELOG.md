# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2023-07-16

### Bug Fixes

- Restrict `/configure` to administrator usage

### Miscellaneous Tasks

- Split providers, variables and outputs in different source files

### Refactor

- Allow `Action::register` to register multiple commands

## [0.1.0] - 2023-07-10

### Bug Fixes

- Match `democracy`'s sea-orm feature flags
- Use unique ids for the action buttons
- Update field `updated_at` before performing the update
- Add missing `WHERE` clause in `invite_poll_update_counters`'s `UPDATE` statement
- Do fetch the number of guild members
- Do not count bot users towards the total of the required votes
- Remove id check from the WHERE clause
- Stop execution if any of the parallel tasks fail
- Invite_poll_with_vote_count should perform a left join
- Represent snowflake ids as strings on the database
- Set all application commands on startup
- Only register actions that create a command
- Clamp the invite poll quorum between 0.0 and 1.0
- Centralize the name of the poll-id field
- Make the main loop infallible
- Typo in the docker builx command
- Add CHANGELOG.md to the staging area after generating it
- Fall back to setting the invite url in the poll embed if the DM could not be sent

### Documentation

- Clean up types and field names in the ER diagram

### Features

- Create a basic `/ping` slash command
- Create the `/invite` command
- Read settings from config and connecto to the database
- Allow specifying the postgresql schema
- Create the InvitePoll table
- Create database entry
- Embed poll id in the generated response
- Create the `InvitePollVoteSubmission` entity
- Create MessageComponentAction::SubmitInvitePollVote
- Create `InvitePoll::find_by_id`
- Fetch poll by encoded id
- Create an entry in the database
- Update counters on `invite_poll` when votes are submitted
- Display results in the form of ascii progressbars
- Use different button styles instead of emojis
- Increase configurability
- Add a status field
- Change embed rendering depending on the poll's status
- Add field `ends_at`
- Create background task that checks for expired polls
- Replace InvitePollStatus with InvitePollOutcome
- Complete poll and send invite privately
- Render progressbars relative to the total amount of votes
- Accept human-like durations like "1 day"
- Re-render the discord message after a poll ends
- Create guild table
- Create `/configure` command
- Configure a maximum amount of 'maybe' votes
- Create a `reason` field
- Implement `Display` as a user tag
- Save both `inviter` and `invitee`

### Miscellaneous Tasks

- Initial commit
- Make the bot crate a workspace member
- Remove `migration` crate
- Remove `/ping` command
- Change migration name to match the created table name
- Remove extraneous comments
- Remove unused imports
- Remove redundant debug! macros
- Configure filetype to `pgsql`
- Derive Debug and sqlx::FromRow
- Remove unused method find_by_id
- Update serenity 0.11.5->0.11.6
- Update sqlx 0.6.3->0.7.0
- Remove unused error variants
- Replace `__self` in async traits with `self`
- Add various emojis
- Populate `description`, `license` and urls
- License democracy under the MIT or Apache 2.0 licenses
- Create git-cliff's default config file
- Set up cargo-release
- Create kubernetes deployment
- Use cargo-release's environment variables
- Reorder Cargo.toml's fields
- Release democracy version 0.1.0

### Refactor

- Update the InvitePoll table's id field to be an UUID
- Switch from sea-orm to sqlx
- Clean up table and attribute names
- Rename `Invite` to `InvitePoll`
- Create `on_application_command_interaction`
- Move poll creation logic inside InvitePoll
- Extract `guild_id` as part of the TryFrom
- Remove discord response generation logic from the entity
- Rename Command to Action
- Rename Invite to CreateInvitePoll
- Split Action in ApplicationCommandAction and MessageComponentAction
- Centralize id text encoding and decoding
- Pass the id as a reference
- Centralize interaction rendering logic
- Avoid unnecessary database loads
- Count votes using a view instead of a trigger
- Rename to `InvitePollVoteCount`
- Declare enums in the module root
- Re-export the poll's information in the "*WithVoteCount" view
- Use discord's color palette
- Create Handler::on_ready
- Create type `InvitePollId`
- Make the connection pool globally statically available
- Clean up the API
- Clean up API
- Simplify discord embed rendering logic
- Extract the background poll handler into its own module
- Reduce amount of 'as' casts
- Centralize poll id extraction logic
- Remove 'maybe' voting option
- Rename vote_success_threshold to invite_poll_quorum
- Extract action into a standalone component
- Extract `CreateInvitePoll` and create responses in `Action::execute`
- Reimplement as an Action
- Remove types, traits and macros from the module root
- Create `MismatchedAction` variant for `ParseActionError`
- Replace `ApplicationCommandAction` and `MessageComponentAction`
- Remove trait field "ID"
- Move into its own module
- Rename `create` to `create_or_update` and take an executor in `find_by_id`
- Take an executor in InvitePoll::close
- Rename `upsert` to `create_or_update`
- Pass executors to `find_by_id` and `find_expired`
- Avoid returning boxed clojures
- Separate outcome and poll status

### Styling

- Format queries
- Display outcome and reason
- Display start and end dates

### Build

- Create Dockerfile

<!-- generated by git-cliff -->
