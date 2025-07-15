✅ Can the unit tests use an in-memory db? - IMPLEMENTED with testcontainers

✅ please add a relays table like mentioned in flurdy's postfix guide - IMPLEMENTED

✅ please add a relocated table like mentioned in flurdy's postfix guide - IMPLEMENTED

✅ please add a clients table that I will describe - IMPLEMENTED

✅ can you check the templates if any hard coded text has been left behind. - IMPLEMENTED

✅ can you check the templates if any message bundle references are missing. - IMPLEMENTED

✅ remove sqllite - IMPLEMENTED

✅ update the dashboard and statistics with the new resources. - IMPLEMENTED

✅ can we add a button to add missing required aliases on the show domain page rows, and missing catchall if desired. - IMPLEMENTED

✅ aliases need to be unique

✅ add a french translation

✅ add a norwegian translation

✅ Prefer a British flag to the US flag in the language drop down, or maybe a hybrid flag.

✅ please tell me why the github workflow CI tests fail

✅ Lets introduce proper authentication. The admin credentials will be stored as config. - IMPLEMENTED with role-based access control

✅ The resources lists should support paging, in case of a lot rows.

✅ UI tests now have Selenium readiness checks and headless browser testing options - IMPLEMENTED

✅ Cleaned up UI tests to use only headless testcontainers approach - IMPLEMENTED

✅  On the dashboard the quick actions needs updating, the domain and backup can merge, some resources are missing etc.
The current dashboard quick actions are a bit repetitive, perhaps invert it.

✅ Update the statistics page with more metrics.

✅ Go over the Rust code to see if there is any duplication we can make cleaner.

✅ Fix Rust code warnings and clippy issues - IMPLEMENTED (88 warnings fixed, build now clean)

✅ When filling in an alias destination we could have an inline search for an existing alias, whilst still support adding unknown destinations. - IMPLEMENTED with HTMX-powered search and comprehensive testing

✅ Create a few longer functional testing journeys

✅ please add support for multiple databases as this will manage several servers - IMPLEMENTED (basic UI and infrastructure)

✅ Multi-database support - IMPLEMENTED:
  - Session-based database selection (store selected database in user session)
  - Update all handlers to use selected database instead of default pool
  - Database selection UI and handler
  - Database selection persistence in session cookies

✅ Multi-database support - remaining steps:
  - Database migration management (run migrations on all configured databases)
  - Show current selected database in navigation/header
  - Add database connection health checks
  - Add database-specific configuration

✅ Add feature toggles, per database, for read only, for no new users, for no new domains. For no password updates. - CONFIG-BASED ONLY

✅ Add German translation - IMPLEMENTED with full i18n support

✅ Put inline css style and scripts in the header into separate files where suitable - IMPLEMENTED

✅ The dashboard stats should be changed to
  - a combined enabled domains and backups total
  - an enabled aliases total
  - an enabled users total
  - The other stats belong and are already in the stats page

✅ In dark mode, there is flash of white for a very brief time when loading new pags which is a bit off-putting - IMPLEMENTED

Add some more reports
   - orphaned alias / users
   - Externally forwarders
   - Domains missing required aliases and no catch-all
   - Check on an alias across all domains

✅ The users list says "disable user" instead of just "disable" on each button row.
✅ In the add and edit user page the buttons have no text as well as some field placeholders - IMPLEMENTED

✅ editing users should not show the password field, instead we should add a separate change password form, and an alternative button to just toggle the change_password field - IMPLEMENTED

There are missing translations for text, headers and buttons for all the user pages.

✅ Add some seed data for relocated, clients and relays as well. - IMPLEMENTED

When filling in an alias mail field or a user id field when entering a @ should trigger a search through domain names as suggestions? Similar to suggestions for alias destination.

Can the database info in the header be a link to database selection, or even a drop down if not messy.

In the show domain page can the existing aliases be shown above the missing ones.

✅ Show per-database feature toggles (read-only, visible to admins) on the config page - IMPLEMENTED with full i18n support

Should we still use a CDN for HTMX and Tailwind?

Lets discuss how we can support different databases with different field names.

Document how we add remote prod databases
  - add remote databases via SSH
  - add remote databases running in AWS EC2 if the app is running in Kubernetes in DigitalOcean
  - how we ensure no migration is run on those

The /static folder should organise its content into "images" etc
