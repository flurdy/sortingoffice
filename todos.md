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

🔄 The resources lists should support paging, in case of a lot rows.

On the dashboard the quick actions needs updating, the domain and backup can merge, some resources are missing etc.

Update the statistics page with more metrics.

Go over the Rust code to see if there is any duplication we can make cleaner.

when filling in an alias destination we could have an inline search for an existing alias, whilst still support adding unknown destinations.

create a few functional testing journeys

please add support for multiple databases as this will manage several servers

Lets discuss how we can support different databases with different field names.

Add feature toggles, per database, for read only, for no new users, for no new domains. For no password updates.

Add Dutch translation

Add German translation

Should we still use a CDN for HTMX and Tailwind?

Put inline css style and scripts in the header into separate files where suitable

The dashboard stats should be changed to
  - a combined enabled domains and backups total
  - an enabled aliases total
  - an enabled users total

The dashboard should have descriptive link to further stats and reports

The current dashboard quick actions are a bit repetitive, perhaps invert it.
