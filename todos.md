# TODO List

## High Priority Epics

## High Priority Minor and bugs 🐛

- ✅ In prod, reports were throwing 500 errors:
  - ✅ Orphan report blew up with a 500
    - Already optimized to O(n+m) with HashSet/HashMap lookups  
    - **FIXED**: Added 10,000 record safety limit with detailed error logging
    - If limit exceeded, logs will show exact breakdown (aliases/users/relays/relocated)
    - Can use "hide_disabled" filter to reduce count before rendering
  - ✅ Domain statistics report blew up with a 500
    - **FIXED**: Rewritten from 6000+ individual queries to just 5 bulk queries
    - Before: For 1000 domains = ~6000 SQL queries (6 per domain)
    - After: 5 queries total + HashMap aggregation in Rust
    - Added comprehensive logging and timing
  - ✅ Recent changes report blew up with a 500 
    - Already limited to 50 items per table (350 max records)
    - **FIXED**: Added detailed timing logs and error handling
    - **ACTION NEEDED**: Add database indexes on `modified` columns for better performance
    - See `/docs/PERFORMANCE_INDEXES.md` for SQL statements to create indexes
  - All fixes ready for production testing with comprehensive logging
  - Logs will now show exact timing and record counts for diagnosis

## Medium Priority Epics

## Medium Priority Minor and bugs 🐛

- ✅ The paging seems not to be on the mx reports anymore? Related to the recent filtering, maybe?

- ✅ In the mx servers report, can it be shown if a domain:
  - ✅ enabled, disabled
  - ✅ normal domain or backup domain 

- ✅ On a prod site, the orphaned report keep blowing up. It has a lot of domain and aliases.
  - Optimized to use HashSet/HashMap lookups instead of individual DB queries per record
  - Performance improved from O(n*m) to O(n+m) where n=records, m=domains
  - Added comprehensive error handling and logging at each stage of report generation
  - Added timing logs to identify bottlenecks (start time, elapsed time, record counts)
  - Added detailed error logging with context at every database operation
  - All unit tests (112/112) pass
  - Ready for production testing - logs will now show exactly where any error occurs

- ✅ On a prod site, the orphaned report still keep blowing up.
  - **ROOT CAUSE**: Database queries were loading ALL aliases/users/relays/relocated without limits
  - On large production sites with 100k+ records, this exhausted memory before safety limit check
  - **FIXED**: Added `MAX_RECORDS_PER_TABLE = 100,000` limit at database level
  - Applied `.limit()` to all 5 table queries (aliases, users, alias_mails, relays, relocated)
  - Added warning logs when limits are hit to inform admins
  - Memory now capped at ~100k records per table instead of unlimited
  - All 118 unit tests pass
  - Ready for production testing 

## Low Priority Epics

- ✅ For paged resources off option to change page size from default 20 to 10 or 50.
  - ✅ Added pagination translation keys to all 7 locales (pagination-page-size, pagination-page-size-10/20/50)
  - ✅ Added page size selector to ALL 6 resource list pages
  - ✅ **UPDATED**: Changed from dropdown to radio toggle for consistency
  - ✅ Domains list page - 3-button toggle: 10 | 20 | 50
  - ✅ Aliases list page - 3-button toggle: 10 | 20 | 50
  - ✅ Users list page - 3-button toggle: 10 | 20 | 50
  - ✅ Relays list page - 3-button toggle: 10 | 20 | 50
  - ✅ Relocated list page - 3-button toggle: 10 | 20 | 50
  - ✅ Clients list page - 3-button toggle: 10 | 20 | 50
  - ✅ Updated all 6 template structs with page size translation fields
  - ✅ Updated all 6 rendering functions to fetch and pass translations
  - ✅ Toggle preserves all query params (search, filters, etc) when changing
  - ✅ Toggle appears in filters section next to enabled/disabled toggle
  - ✅ Uses same radio-toggle CSS as enabled/disabled filter for consistency
  - ✅ All 118 unit tests pass ✅
  - ✅ Code formatted and compiled successfully

- ✅ In show alias, at the bottom replicate the Alias across domains report for that alias

## Low Priority Minor and bugs 🐛 

- ✅ On list domains page, can the include subdomains checkbox also be a toggle to align it with the filters on that page.
  - ✅ Added subdomain filter translation keys to all 7 locales
  - ✅ Converted checkbox to 2-button radio toggle: "All Domains | Exclude Subdomains"
  - ✅ Toggle appears with label "Subdomains" for consistency with other filters
  - ✅ Uses same radio-toggle CSS as page size and status filters
  - ✅ Both options now use blue color (not green/red) for consistency with page size toggle
  - ✅ Toggle preserves all query params (search, filters, etc) when changing
  - ✅ All 118 unit tests pass
  - ✅ Code formatted and compiled successfully

- On the list domains page, clicking to the next page of backup domains seems smooth, probably htmx? But clicking on the next page for domains seems to refresh the whole page?

- ✅ In show alias, if the alias is a catch all, please do not include the alias occurrences report.

- ✅ On the domain statistics report, remove the quota columns. Add relays and relocated.
  - With enabled and disabled shown as well?

## 🙈 KNOWN ISSUES

* Adding alias/user full email as domain field in a backup domain is an edge case 

## ⏩ Postponed epics

- Have a cached/timebased undo feature for deletion
