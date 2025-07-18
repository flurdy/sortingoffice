# Application title and branding
app-title = Sorting Office
app-subtitle = Mail Server Administration Tool

# Navigation
nav-dashboard = Dashboard
nav-domains = Domains
nav-aliases = Aliases
nav-users = Users
nav-relays = Relays
nav-relocated = Relocated
nav-clients = Clients
nav-statistics = Statistics
nav-reports = Reports
nav-config = Configuration
nav-database = Database Selection
nav-about = About
nav-logout = Logout

# UI Controls
theme-toggle = Toggle theme
language-selector = Language
language-english = English
language-spanish = Español
language-french = Français
language-norwegian = Norsk

# About Page
about-title = About Sorting Office
about-subtitle = A comprehensive mail server administration tool for managing domains, users, aliases, and backups.
about-what-is-title = What is Sorting Office?
about-what-is-p1 = Sorting Office is a web-based administration interface for managing a complete mail server setup. It provides an intuitive way to manage domains, users, email aliases, and backup configurations for a Postfix and Dovecot mail server environment.
about-what-is-p2 = Built with Rust and modern web technologies, Sorting Office offers a secure, fast, and reliable way to administer your mail server without needing to manually edit configuration files.
about-features-title = Key Features
about-feature-domain-management = Domain Management
about-feature-domain-management-desc = Add, edit, and manage mail domains with ease
about-feature-user-management = User Management
about-feature-user-management-desc = Create and manage email users and accounts
about-feature-alias-management = Alias Management
about-feature-alias-management-desc = Set up email aliases and forwarding rules
about-feature-backup-configuration = Backup Configuration
about-feature-backup-configuration-desc = Configure and manage backup MX servers
about-feature-statistics-dashboard = Statistics Dashboard
about-feature-statistics-dashboard-desc = Monitor your mail server usage and statistics
about-feature-dark-mode-support = Dark Mode Support
about-feature-dark-mode-support-desc = Modern UI with dark mode for comfortable use
about-technology-stack-title = Technology Stack
about-backend = Backend
about-backend-desc = Rust with Axum web framework
about-database = Database
about-database-desc = MySQL with Diesel ORM
about-frontend = Frontend
about-frontend-desc = HTML templates with Tailwind CSS
about-templating = Templating
about-templating-desc = Askama template engine
about-mail-server = Mail Server
about-mail-server-desc = Postfix & Dovecot
about-deployment = Deployment
about-deployment-desc = Docker containerization
about-based-on-flurdy-title = Based on flurdy's Complete Mail Server Guide
about-based-on-flurdy-desc = This administration tool is designed to work with the comprehensive mail server setup guide created by flurdy, which covers Postfix, Dovecot, SpamAssassin, and more.
about-read-guide = Read the complete setup guide →
about-github-project-title = GitHub Project
about-open-source = Open Source
about-open-source-desc = Sorting Office is an open-source project hosted on GitHub under the MIT license.
about-view-repository = View Repository
about-view-repository-desc = Browse source code and documentation
about-report-issues = Report Issues
about-report-issues-desc = Bug reports and feature requests
about-pull-requests = Pull Requests
about-pull-requests-desc = Contribute to the project
about-readme = README
about-readme-desc = Project documentation and setup
about-version-information = Version Information
about-project-details = Project Details
about-version = Version
about-license = License
about-maintainer = Maintainer

# Common actions
action-edit = Edit
action-delete = Delete
action-view = View
action-save = Save
action-cancel = Cancel
action-enable = Enable
action-disable = Disable

# Status indicators
status-active = Active
status-inactive = Inactive
status-enabled = Enabled
status-disabled = Disabled

# Dashboard
dashboard-title = Dashboard
dashboard-description = Overview of your mail server statistics and quick actions.
dashboard-total-domains = Total Domains
dashboard-total-backups = Total Backups
dashboard-total-aliases = Total Aliases
dashboard-total-users = Total Users
dashboard-total-relays = Total Relays
dashboard-total-relocated = Total Relocated
dashboard-total-clients = Total Clients
dashboard-enabled-domains-and-backups = Enabled Domains & Backups
dashboard-enabled-aliases = Enabled Aliases
dashboard-enabled-users = Enabled Users
dashboard-quick-actions = Quick Actions
dashboard-primary-actions = Primary Actions
dashboard-advanced-management = Advanced Management
dashboard-analytics-reports = Analytics & Reports
dashboard-help-resources = Help & Resources

# Quick action cards
quick-action-manage-domains = Manage Domains
quick-action-manage-domains-desc = Add, edit, or remove domains
quick-action-manage-backups = Manage Backups
quick-action-manage-backups-desc = Add, edit, or remove backups
quick-action-manage-domains-and-backups = Domains & Backups
quick-action-manage-domains-and-backups-desc = Manage domains and backup servers
quick-action-manage-email = Email Management
quick-action-manage-email-desc = Manage email aliases and forwarding
quick-action-manage-aliases = Manage Aliases
quick-action-manage-aliases-desc = Add, edit, or remove email aliases
quick-action-manage-users = Manage Users
quick-action-manage-users-desc = Add, edit, or remove users
quick-action-manage-clients = Manage Clients
quick-action-manage-clients-desc = Add, edit, or remove client access controls
quick-action-manage-relays = Manage Relays
quick-action-manage-relocated = Manage Relocated
quick-action-manage-config = Configuration
quick-action-manage-config-desc = System settings and preferences
quick-action-view-statistics = View Statistics
quick-action-view-statistics-desc = System metrics and analytics
quick-action-view-reports = View Reports
quick-action-view-reports-desc = Detailed reports and analysis

# Resource labels
resource-domains = domains
resource-backups = backups
resource-aliases = aliases
resource-users = users
resource-relays = relays
resource-relocated = relocated
resource-clients = clients

# Help section
help-title = Need help setting up your mail server?
help-description = This admin tool is based on flurdy's comprehensive guide for setting up a complete mail server with Postfix, Dovecot, and more.
help-read-guide = Read the complete setup guide →

# Domains
domains-title = Domains
domains-description = Manage your mail server domains and their settings.
domains-add = Add Domain
domains-table-header-domain = Domain
domains-table-header-enabled = Enabled
domains-table-header-actions = Actions
domains-empty-title = No domains
domains-empty-description = Get started by creating a new domain.
domains-show-title = Domain
domains-new-domain = New Domain
domains-edit-domain = Edit Domain
domains-view-edit-settings = View and edit domain settings.
domains-back-to-domains = Back to Domains
domains-domain-information = Domain Information
domains-domain-details = Domain details and configuration.
domains-domain-name = Domain name
domains-transport = Transport
domains-status = Status
domains-created = Created
domains-modified = Modified
domains-edit-domain-button = Edit Domain
domains-enable-domain = Enable Domain
domains-disable-domain = Disable Domain
domains-delete-domain = Delete Domain
domains-delete-confirm = Are you sure you want to delete this domain?
domains-not-found = Domain not found
domains-add-missing-required-aliases-button = Add alias
domains-add-catch-all-button = Add alias
domains-add-alias-button = Add new alias
domains-no-catch-all-message = No catch-all alias configured for this domain

# Backups
backups-title = Backups
backups-description = Manage your backup mail servers and their settings.
backups-add = Add Backup
backups-table-header-domain = Domain
backups-table-header-transport = Transport
backups-table-header-enabled = Enabled
backups-table-header-actions = Actions
backups-view = View
backups-enable = Enable
backups-disable = Disable
backups-empty-no-backup-servers = No backup servers
backups-empty-get-started = Get started by creating a new backup server.
backups-show-title = Backup
backups-view-edit-settings = View and edit backup server settings.
backups-backup-information = Backup Information
backups-backup-details = Backup server details and configuration.
backups-domain = Domain
backups-transport = Transport
backups-status = Status
backups-created = Created
backups-modified = Modified
backups-edit-backup = Edit Backup
backups-enable-backup = Enable Backup
backups-disable-backup = Disable Backup
backups-delete-backup = Delete Backup
backups-delete-confirm = Are you sure you want to delete this backup?
backups-form-error = Error
backups-form-domain = Domain
backups-form-transport = Transport
backups-form-active = Active
backups-placeholder-domain = backup.example.com
backups-placeholder-transport = smtp:[]
backups-tooltip-domain = The domain name for the backup server
backups-tooltip-transport = Transport configuration for the backup server (e.g., smtp:[] for local delivery)
backups-tooltip-active = Enable this backup server
backups-cancel = Cancel
backups-create-backup = Create Backup
backups-update-backup = Update Backup
backups-new-backup = New Backup
backups-edit-backup-title = Edit Backup

# Aliases
aliases-title = Aliases
aliases-description = Manage your email aliases and forwarding rules.
aliases-add = Add Alias
aliases-table-header-destination = Destination
aliases-table-header-enabled = Enabled
aliases-enable-alias = Enable
aliases-disable-alias = Disable
aliases-view-edit-settings = View and edit alias settings.
aliases-back-to-aliases = Back to Aliases
aliases-alias-information = Alias Information
aliases-alias-details = Alias details and configuration.
aliases-mail = Mail
aliases-forward-to = Forward To
aliases-status = Status
aliases-created = Created
aliases-modified = Modified
aliases-edit-alias-button = Edit Alias
aliases-enable-alias-button = Enable Alias
aliases-disable-alias-button = Disable Alias
aliases-delete-alias = Delete Alias
aliases-delete-confirm = Are you sure you want to delete this alias?
aliases-edit-alias = Edit Alias
aliases-new-alias = New Alias
aliases-form-error = Error
aliases-mail-address = Mail Address
aliases-destination = Destination
aliases-placeholder-mail = alias@example.com
aliases-placeholder-destination = destination@example.com
aliases-tooltip-mail = The email address for the alias (e.g., alias@example.com)
aliases-tooltip-destination = The destination email address where mail will be forwarded
aliases-active = Active
aliases-tooltip-active = Enable this alias for mail forwarding
aliases-cancel = Cancel
aliases-update-alias = Update Alias
aliases-create-alias = Create Alias
aliases-search-no-results = No matching aliases found
aliases-search-select = Click to select

# Users
users-title = Users
users-description = Manage your mail server users and their accounts.
users-add = Add User
users-table-header-username = Username
users-table-header-enabled = Enabled
users-table-header-actions = Actions
users-empty-title = No users
users-empty-description = Get started by creating a new user.
users-add-title = Add User
users-edit-title = Edit User
users-show-title = User
users-view = View
users-enable = Enable
users-disable = Disable
users-show-user-title = User
users-view-edit-settings = View and edit user settings.
users-back-to-users = Back to Users
users-user-information = User Information
users-user-details = User details and configuration.
users-user-id = User ID
users-status = Status
users-created = Created
users-modified = Modified
users-edit-user = Edit User
users-enable-user = Enable
users-disable-user = Disable
users-delete-user = Delete User
users-delete-confirm = Are you sure you want to delete this user?
users-new-user = New User
users-edit-user-title = Edit User
users-form-user-id = User ID
users-form-password = Password
users-form-name = Mailbox name
users-form-active = Active
users-placeholder-user-email = user@example.com
users-placeholder-name = mailbox name
users-maildir = Maildir path
users-tooltip-maildir = The root folder for this user's mailbox (relative to mail storage root)
users-placeholder-maildir = e.g. example.com/user/
users-tooltip-user-id = The email address for the user account (e.g., user@example.com)
users-tooltip-password = The password for the user account (leave empty to keep existing password when editing)
users-tooltip-name = The name of the user's mailbox folder
users-tooltip-active = Enable this user account
users-cancel = Cancel
users-create-user = Create User
users-update-user = Update User
users-change-password = Require password change
users-change-password-tooltip = Require password change on next login
users-placeholder-password = Enter new password (leave empty to keep existing)
users-password-change-required-label = Password Change Required
users-password-change-required-yes = Yes
users-password-change-required-no = No
users-password-management-title = Password Management
users-change-password-button = Change Password
users-require-password-change-button = Require Password Change
users-change-password-title = Change Password for { $name }
users-new-password-label = New Password
users-new-password-placeholder = Enter new password
users-confirm-password-label = Confirm Password
users-confirm-password-placeholder = Confirm new password
users-cancel-button = Cancel

# Form actions
form-create-user = Create User
form-update-user = Update User
form-cancel = Cancel
form-placeholder-username = user@example.com
form-placeholder-password = Enter new password (leave empty to keep existing)
form-placeholder-name = John Doe
form-tooltip-username = The email address for the user account (e.g., user@example.com)
form-tooltip-password = The password for the user account (leave empty to keep existing password when editing)
form-tooltip-name = The display name for the user (e.g., John Doe)
form-tooltip-enable = Enable this user account

# Statistics
stats-title = Statistics
stats-description = View detailed statistics about your mail server.
stats-system-overview = System Overview
stats-system-description = System-wide and per-domain statistics for your mail server.
stats-total-domains = Total Domains
stats-total-backups = Total Backups
stats-total-aliases = Total Aliases
stats-total-users = Total Users
stats-total-relays = Total Relays
stats-total-relocated = Total Relocated
stats-total-clients = Total Clients
stats-domain-statistics = Domain Statistics
stats-table-header-domain = Domain
stats-table-header-users = Users
stats-table-header-aliases = Aliases
stats-table-header-total-quota = Total Quota
stats-table-header-used-quota = Used Quota
stats-quota-usage-title = Quota Usage
stats-quota-usage-overview = Storage Usage Overview
stats-quota-usage-description = Monitor disk space usage across all domains and users
stats-quota-usage-percentage = Usage Percentage
stats-quota-total = Total Quota
stats-quota-used = Used Quota
stats-recent-activity-title = Recent Activity
stats-recent-domains = Recent Domains
stats-recent-users = Recent Users
stats-recent-aliases = Recent Aliases
stats-recent-backups = Recent Backups
stats-recent-relays = Recent Relays
stats-recent-relocated = Recent Relocated
stats-recent-clients = Recent Clients

# Forms
form-domain = Domain
form-transport = Transport
form-username = Username
form-password = Password

# Form validation
validation-domain-required = Domain name is required. Please enter a valid domain name.
validation-username-required = Username is required.

# Error messages
error-unexpected = An unexpected error occurred. Please try again.
error-duplicate-domain = A domain with this name already exists.
error-duplicate-backup = A backup server for domain '{ $domain }' already exists.
error-duplicate-user = A user with this username already exists.
error-constraint-violation = The data does not meet the required constraints. Please check your input.
error-operation-not-allowed = This operation is not allowed on the current database due to restrictions.

# Success messages

# Theme
theme-toggle = Toggle theme

# Login
login-title = Sign in to Sorting Office
login-user-id = User ID
login-password = Password
login-sign-in = Sign in
login-error = Invalid username or password
login-error-invalid-credentials = Invalid username or password. Please try again.
login-error-empty-fields = Please enter both username and password.

# Roles and Permissions

# About
about-title = About Sorting Office
about-what-is-title = What is Sorting Office?
about-what-is-p1 = Sorting Office is a web-based administration interface for managing a complete mail server setup. It provides an intuitive way to manage domains, users, email aliases, and backup configurations for a Postfix and Dovecot mail server environment.
about-what-is-p2 = Built with Rust and modern web technologies, Sorting Office offers a secure, fast, and reliable way to administer your mail server without needing to manually edit configuration files.
about-features-title = Key Features
about-feature-domain-management = Domain Management
about-feature-domain-management-desc = Add, edit, and manage mail domains with ease
about-feature-user-management = User Management
about-feature-user-management-desc = Create and manage email users and accounts
about-feature-alias-management = Alias Management
about-feature-alias-management-desc = Set up email aliases and forwarding rules
about-feature-backup-configuration = Backup Configuration
about-feature-backup-configuration-desc = Configure and manage backup MX servers
about-feature-statistics-dashboard = Statistics Dashboard
about-feature-statistics-dashboard-desc = Monitor your mail server usage and statistics
about-technology-stack-title = Technology Stack
about-backend = Backend
about-backend-desc = Rust with Axum web framework
about-database = Database
about-database-desc = MySQL with Diesel ORM
about-frontend = Frontend
about-frontend-desc = HTML templates with Tailwind CSS
about-templating = Templating
about-templating-desc = Askama template engine
about-mail-server = Mail Server
about-mail-server-desc = Postfix & Dovecot
about-deployment = Deployment
about-deployment-desc = Docker containerization
about-github-project-title = GitHub Project
about-open-source = Open Source
about-open-source-desc = Sorting Office is an open-source project hosted on GitHub under the MIT license.
about-view-repository = View Repository
about-view-repository-desc = Browse source code and documentation
about-report-issues = Report Issues
about-report-issues-desc = Bug reports and feature requests
about-pull-requests = Pull Requests
about-pull-requests-desc = Contribute to the project
about-readme = README
about-readme-desc = Project documentation and setup
about-version-information = Version Information
about-project-details = Project Details
about-version = Version
about-license = License
about-maintainer = Maintainer

# Forms
form-error = Error
form-cancel = Cancel
form-create = Create
form-update = Update
form-active = Active
form-placeholder-domain = example.com
form-placeholder-transport = virtual
form-tooltip-domain = The domain name (e.g., example.com)
form-tooltip-transport = Transport configuration (e.g., virtual for virtual domains)
form-tooltip-enable = Enable this domain
form-create-domain = Create Domain
form-update-domain = Update Domain

# Reports
reports-domain-header = "Domain"
reports-destination-header = "Destination"
reports-required-aliases-header = "Required Aliases"
reports-missing-aliases-header = "Missing Required Aliases"
reports-missing-required-aliases-header = "Missing Required Aliases"
reports-missing-common-aliases-header = "Missing Common Aliases"
reports-add-missing-required-alias-button = "Add alias"
reports-add-common-alias-button = "Add alias"
reports-add-catch-all-button = "Add alias"
reports-alias-report-title = "Alias Report"
reports-alias-report-description = "Overview of catch-all, required, and common aliases for this domain"
reports-missing-required-alias-header = "Missing Required Aliases"
reports-existing-aliases-header = "Existing Aliases"
reports-no-catch-all-message = "No catch-all alias configured for this domain"
reports-mail-header = "Email"
reports-status-header = "Status"
reports-enabled-header = "Enabled"
reports-actions-header = "Actions"
reports-no-required-aliases = "No required aliases found for this domain"
reports-no-missing-aliases = "No missing required aliases for this domain"
reports-catch-all-header = Catch-All Alias
reports-no-domains = "No domains found"
reports-no-domains-description = "No domains are configured in the system"

# Configuration
config-title = "Configuration"
config-description = "Manage required aliases and domain-specific overrides."
config-required-aliases-header = "Required Aliases"
config-common-aliases-header = "Common Aliases"
config-domain-overrides-header = "Domain Overrides"
config-save-button = "Save Configuration"
config-cancel-button = "Cancel"
config-add-required-alias-button = "Add Required Alias"
config-add-common-alias-button = "Add Common Alias"
config-remove-alias-button = "Remove"
config-promote-button = "Promote to Required"
config-demote-button = "Demote to Common"
config-required-aliases-description = "These aliases are essential for email standards compliance (RFC requirements)."
config-common-aliases-description = "These aliases are commonly expected by users and services but not strictly required."
config-domain-overrides-description = "Override required aliases for specific domains."
config-add-domain-override-button = "Add Domain Override"
config-remove-domain-button = "Remove Domain"
config-required-aliases-label = "Required Aliases"
config-common-aliases-label = "Common Aliases"
config-remove-button = "Remove"
config-add-alias-button = "Add Alias"
config-placeholder-required-alias = "Enter new required alias (e.g., postmaster)"
config-placeholder-common-alias = "Enter new common alias (e.g., admin)"
config-placeholder-domain = "Enter domain (e.g., example.com)"
config-placeholder-domain-alias = "Enter alias for {domain}"

# Global Feature Toggles
config-global-features-header = "Global Feature Toggles"
config-global-features-description = "These settings apply to all databases unless overridden by database-specific settings."
config-feature-read-only = "Read Only"
config-feature-no-new-users = "No New Users"
config-feature-no-new-domains = "No New Domains"
config-feature-no-password-updates = "No Password Updates"
config-feature-database-disabled = "Database Disabled"
config-status-enabled = "Enabled"
config-status-disabled = "Disabled"

# Database Feature Toggles
config-database-features-header = "Database Feature Toggles"
config-database-features-description = "Database-specific feature restrictions. These override global settings when enabled."
config-database-disabled-badge = "Disabled"

# Matrix Report
reports-matrix-title = "Domain Alias Matrix Report"
reports-matrix-description = "Comprehensive overview of all domains and their alias status with visual indicators"
reports-status-present = "Present and enabled"
reports-status-missing = "Missing"
reports-status-disabled = "Present but disabled"
reports-legend-title = "Status Legend"

# Reports List
reports-list-title = "Reports"
reports-list-description = "View and analyze mail server data with comprehensive reports"
reports-view-report = "View Report"

# Matrix Report
reports-matrix-title = "Domain Alias Matrix Report"
reports-matrix-description = "Comprehensive overview of all domains and their alias status with visual indicators"

# Orphaned Aliases & Users Report
reports-orphaned-aliases-title = "Orphaned Aliases & Users Report"
reports-orphaned-aliases-description = "Find aliases that reference non-existent users or domains, and users that exist but have no associated aliases"

# External Forwarders Report
reports-external-forwarders-title = "External Forwarders Report"
reports-external-forwarders-description = "Find aliases that forward to external domains"

# Missing Aliases Report

# Alias Cross-Domain Search Report
reports-alias-cross-domain-title = "Alias Cross-Domain Search"
reports-alias-cross-domain-description = "Search for aliases across all domains"

# Cross-Database Domain Matrix Report
reports-cross-db-matrix-title = "Cross-Database Domain Matrix"
reports-cross-db-matrix-description = "Matrix showing domains present across all databases, whether as primary or backup domains"
reports-database-header = "Database"
reports-primary-domain = "Domain"
reports-backup-domain = "Backup"
reports-not-present = "Not Present"

# Cross-Database User Distribution Report
reports-cross-db-user-distribution-title = "Cross-Database User Distribution"
reports-cross-db-user-distribution-description = "Shows how users are distributed across databases, highlighting duplicates and single-database users"
reports-user-header = "User"
reports-present = "Present"
reports-no-users = "No users found"
reports-no-users-description = "No users are configured in the system"

# Cross-Database Feature Toggle Report
reports-cross-db-feature-toggle-title = "Cross-Database Feature Toggle Compliance"
reports-cross-db-feature-toggle-description = "Shows feature toggle settings across all databases and compliance with organizational policies"
reports-read-only = "Read Only"
reports-no-new-users = "No New Users"
reports-no-new-domains = "No New Domains"
reports-no-password-updates = "No Password Updates"
reports-database-status-header = "Database Status"
reports-enabled = "Enabled"
reports-disabled = "Disabled"

# Cross-Database Migration Report
reports-cross-db-migration-title = "Cross-Database Migration Status"
reports-cross-db-migration-description = "Tracks migration status across all databases and identifies which need updates"
reports-status-header = "Status"
reports-last-migration-header = "Last Migration"
reports-migration-count-header = "Migration Count"

# Domain Alias Report
domains-alias-report-title = "Alias Report"
domains-alias-report-description = "Overview of all aliases for this domain, including catch-all, required, and common aliases"
domains-existing-aliases-header = "Existing Aliases"

# Relays
relays-title = Relays
relays-add = Add Relay
relays-list-description = Manage relay recipients and their status for Postfix configuration.
relays-table-header-recipient = Recipient
relays-table-header-status = Status
relays-table-header-enabled = Enabled
relays-table-header-actions = Actions
relays-empty-title = No relays
relays-empty-description = Get started by creating a new relay recipient.
relays-add-title = Add Relay
relays-show-title = Relay
relays-show-title-label = Relay
relays-view-edit-settings = View and edit relay settings.
relays-info-title = Relay Information
relays-info-description = Details about this relay configuration.
relays-field-id = ID
relays-field-recipient = Recipient
relays-field-status = Status
relays-field-enabled = Enabled
relays-field-created = Created
relays-field-modified = Modified
relays-delete-confirm = Are you sure you want to delete this relay?
relays-placeholder-recipient = Enter recipient address
relays-placeholder-status = Enter status code
relays-field-recipient-help = The recipient address for this relay (e.g., user@example.com)
relays-field-status-help = The status code for this relay (e.g., active, disabled, etc.)

# Relocated
relocated-title = Relocated
relocated-add = Add Relocated
relocated-list-description = Manage email address relocations for Postfix configuration.
relocated-table-header-old-address = Old Address
relocated-table-header-new-address = New Address
relocated-table-header-enabled = Enabled
relocated-table-header-actions = Actions
relocated-empty-title = No relocated addresses
relocated-empty-description = Get started by creating a new relocated address.
relocated-add-title = Add Relocated
relocated-edit-title = Edit Relocated
relocated-show-title = Relocated
relocated-view-edit-settings = View and edit relocated address settings.
relocated-back-to-list = Back to Relocated
relocated-info-title = Relocated Information
relocated-info-description = Details about this relocated address configuration.
relocated-field-id = ID
relocated-field-old-address = Old Address
relocated-field-new-address = New Address
relocated-field-enabled = Enabled
relocated-field-created = Created
relocated-field-modified = Modified
relocated-delete-confirm = Are you sure you want to delete this relocated address?
relocated-placeholder-old-address = Enter old email address
relocated-placeholder-new-address = Enter new email address
relocated-field-old-address-help = The old email address that should be relocated (e.g., olduser@example.com)
relocated-field-new-address-help = The new email address where mail should be sent (e.g., newuser@example.com)
relocated-new-relocated = New Relocated Address
relocated-edit-relocated = Edit Relocated Address
relocated-not-found = Relocated address not found
relocated-create-error = Failed to create relocated address

# Clients
clients-title = Clients
clients-description = Manage client access controls for SMTP authentication.
clients-add = Add Client
clients-table-header-client = Client
clients-table-header-status = Status
clients-table-header-actions = Actions
clients-empty-title = No clients
clients-empty-description = Get started by creating a new client access control.
clients-add-title = Add Client
clients-edit-title = Edit Client
clients-show-title = Client
clients-view-edit-settings = View and edit client access control settings.
clients-back-to-clients = Back to Clients
clients-info-title = Client Information
clients-info-description = Details about this client access control configuration.
clients-field-client = Client
clients-field-status = Status
clients-field-created = Created
clients-field-updated = Last Updated
clients-action-edit = Edit
clients-action-delete = Delete
clients-action-view = View
clients-delete-confirm = Are you sure you want to delete this client?
clients-placeholder-client = Enter client name or IP
clients-field-client-help = The client name, IP address, or identifier for access control
clients-field-status-help = Whether this client is allowed or blocked
clients-action-cancel = Cancel
clients-action-save = Save
clients-not-found = Client not found
clients-status-allowed = Allowed
clients-status-blocked = Blocked
clients-field-enabled = Enabled
clients-field-enabled-help = Is this client enabled?
clients-table-header-enabled = Enabled
clients-enabled-yes = Yes
clients-enabled-no = No
clients-action-enable = Enable
clients-action-disable = Disable
clients-status-enabled = Enabled
clients-status-disabled = Disabled

# Domain Search
domains-search-no-results = No domains found
domains-search-select = Select a domain
