# TODO List

## High Priority Epics

## High Priority Minor and bugs 🐛

- ✅ recent changes report seem to blow up with a weird 500 that does not show an error page

## Medium Priority Epics

## Medium Priority Minor and bugs 🐛

- ✅ The paging seems not to be on the mx reports anymore? Related to the recent filtering, maybe?

- ✅ Not found aliases is pure text, not styled 404 error page

- ✅ Need to test alias destination containing multiple emails separated by a comma.

- ✅ In the mx servers report, can there be some filter buttons/checkboxes at the top to:
  - ✅ Exclude disabled domains
  - ✅ Exclude subdomains
  - ✅ And filter by the mx status, e.g only show non-compliant etc.

- ✅ In the mx servers report, can it be shown if a domain:
  - ✅ enabled, disabled
  - ✅ normal domain or backup domain

- ✅ In the mx servers report, if on a paged result, switching to another db with less domains may result in an empty result page without paging buttons.

- ✅ In show backup domain page, users and aliases are at the top, they should match the order that is in show domain. 

- ✅ In the list domains page, on a backup domain row clicking disable does nothing.
  - ✅ Console says: 404 for /domain_backup/102/toggle-list

## Low Priority Epics

- ✅ Add html head title and description to all the pages. So that tabs can be distinguished when not wide E.g
   - ✅ show domain could have: 'DOMAINNAME domain at DB db - Sorting Office' 

- ✅ A report to check if any domains mx settings are not in a list of servers. 
  - Basically to check if some domains are not pointing to these mail servers.
  - May need to add optional servers name (e.g mail.example.com) to DBs in the config, to compare with?

- ✅ In show alias, at the bottom replicate the Alias across domains report for that alias

- Can the orphaned report also check relays and relocated entries.

## Low Priority Minor and bugs 🐛 

- ✅ Like in some of the reports, can the we add filters to the domain and backup domain lists? 
   - ✅ enabled/disabled
   - ✅ subdomain

- In show alias, on the domain row, can we add a tiny icon if the domain is enabled or not

- In show alias, if the alias is a catch all, please do not include the alias occurrences report.

- ✅ In show domain the disable domain button should not be blue? 
  - ✅ And in all other resources pages

- ✅ Add a whois lookup under or within the DNS section
  - ✅ As it may be a large blob of text, it may need to be collapsible

- ✅ On the cross-database user distribution, can the user be a link to the show user page if present in the current db.

- ✅ In the Orphaned aliases and users report add a button to toggle filtering out disabled resources, and some way to flag that the domain may be disabled as well

- ✅ On the domain statistics report, remove the quota columns. Add relays and relocated.
  - With enabled and disabled shown as well?

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
