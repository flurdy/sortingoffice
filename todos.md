✅ Can the integration tests be a separate call like make test-unit and test-ui? And not part of test-unit. But part of make test-all. - IMPLEMENTED

✅ Some resource pages (backups list, users list) have enabled as active/inactive not enabled/disabled
- in dark mode the colouring of these are inconsistent in some resource lists. - IMPLEMENTED

✅ In the run_tests.sh does run_unit_tests actually test anything different than run_integration_tests? - IMPLEMENTED
- No, they were running the same tests
- Fixed by:
  - Moving unit tests into source files (src/*.rs)
  - Moving integration tests to tests/ directory
  - Updating run_tests.sh to use --lib for unit tests and --test for integration tests
  - Updating Makefile to properly separate test types

Provide some timings of the unit, integration, ui-headless and ui-containerized.

Why then compiling is there multiple axum versions? 

In the run_tests.sh does run_ui_containerized which uses databases in testcontainers, but then uses the running app which probably don't use the testcontainers db? Or am I missing something?

✅ If a domain has a catchall then the required and common aliases are not missing in the alias reports. - IMPLEMENTED

✅ Can we reorder the English messages.ftl, there seems to be two about sections etc, it would be good if they where joined. And the same for other sections. - IMPLEMENTED

✅ Improve theme slightly - IMPLEMENTED
- add a subtle gray background where applicable
- still support the dark mode

Add wizard for onboarding a set of new domains, maybe with common aliases and destinations across all the new domains, maybe copy existing domain+aliases.

First column of aliases list (mail) is missing translations. So is the alias-empty-title and description.

Like the users table where there is a domain column which is extracted from the id, can we add a similar domain column to the aliases list, extracted from the mail field?

Can the mail, destination and domain column header be clickable in the aliases to sort by that column. If clicking domain, it will secondary sort by mail.

Any way we can ensure no seeding happens on prod databases?

Any way we can block migrations on prod databases?

Can we make certain tables optional? Relays, relocated and clients are not in every db.

On the configuration page there is functionality to add and remove, promote and demote, required and common aliases. We do not need this. Configuring it in config.toml offline is enough.

For clients and relays, the status is either OK, or REJECT.

In the UI I seem unable to edit a relay and relocated.
