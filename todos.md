# TODO List

## High Priority Epics

- ✅ On the list domains page, the backups list also needs to be paged independently of the domains list.
  - ✅ We do not want a separate list backup domains page.
  - ✅ We need to remember which page the domains list and the backup domains list is, when clicking on the paging buttons for both the domains and backup domains list.
  - ✅ Since the SQL lookups are eventually cached this should not be much of further delay in rendering times.

## High Priority Minor and bugs 🐛

- ✅ For some domains I get this error in the logs: [DEBUG] Rendering error page with title: domains-not-found-title, message: domains-not-found-message

## Medium Priority Epics

- ✅ Can we make delete resource only be clickable if a resource is disabled

## Medium Priority Minor and bugs 🐛

- ✅ Not found aliases is pure text, not styled 404 error page

- ✅ Need to test alias destination containing multiple emails separated by a comma.

- ✅ In show domain there is no add user button in the users section

- ✅ on the show user page there should be a link to the domain, like in show alias

- ✅ on the show relay page there should be a link to the domain, like in show alias

- ✅ Converting a domain to a backup domain when finished redirects to the new backup domain, but the path prefix is wrong.

- ✅ At an API level can be we also disable/return an error if any edits/toggles are attempted when that db/global is read only.
  - ✅ e.g. toggling htmx post request to disable/enable resources
  - ✅ This also includes request to GET pages to show add or edit resource when read only.

## Low Priority Epics

- ✅ On a show backup domain page add a button to change it from a backup domain to a normal domain.
  - ✅ And a button to change from domain to backup domain

- ✅ A report to check if any domains mx settings are not in a list of servers. 
  - Basically to check if some domains are not pointing to these mail servers.
  - May need to add optional servers name (e.g mail.example.com) to DBs in the config, to compare with?

- ✅ In show alias, at the bottom replicate the Alias across domains report for that alias

## Low Priority Minor and bugs 🐛 

- ✅ On the cross-database user distribution, can the user be a link to the show user page if present in the current db.

- ✅ In the Orphaned aliases and users report add a button to toggle filtering out disabled resources, and some way to flag that the domain may be disabled as well

- ✅ On the domain statistics report, remove the quota columns. Add relays and relocated.
  - With enabled and disabled shown as well?

- ✅ Add i18n translations for disabled delete button tooltips

- ✅ Long strings in a domain's DKIM or DMARC sections are not wrapped as expected and changes the width of the page.

- ✅ if show domain is for a missing domain, there is an error in the log but just a blank page shown, with correct header and sidebar.
  - ✅ Applied the same fix to all other missing resource pages (users, aliases, relays, backups, relocated, clients)
  - ✅ Not found Alias Still shows unstyled page http://localhost:3000/aliases/85 
  - ✅ Same for relay
  - ✅ Not found domain, user, relocated and client is styled

## 🙈 KNOWN ISSUES

* Adding alias/user full email as domain field in a backup domain is an edge case 

## ⏩ Postponed epics

- Have a cached/timebased undo feature for deletion

- Add a remove domain wizard. Postponed for now.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests
