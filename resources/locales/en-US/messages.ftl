# Application title and branding
app-title = Sorting Office
app-subtitle = Mail Server Administration Tool

# Navigation
nav-dashboard = Dashboard
nav-domains = Domains
nav-backups = Backups
nav-aliases = Aliases
nav-users = Users
nav-statistics = Statistics
nav-about = About
nav-logout = Logout

# UI Controls
theme-toggle = Toggle theme
language-selector = Language
language-english = English
language-spanish = Español

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
action-add = Add
action-edit = Edit
action-delete = Delete
action-view = View
action-save = Save
action-cancel = Cancel
action-back = Back
action-enable = Enable
action-disable = Disable
action-toggle = Toggle

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
dashboard-quick-actions = Quick Actions
dashboard-help-resources = Help & Resources

# Quick action cards
quick-action-manage-domains = Manage Domains
quick-action-manage-domains-desc = Add, edit, or remove domains
quick-action-manage-backups = Manage Backups
quick-action-manage-backups-desc = Add, edit, or remove backups
quick-action-manage-aliases = Manage Aliases
quick-action-manage-aliases-desc = Add, edit, or remove email aliases
quick-action-manage-users = Manage Users
quick-action-manage-users-desc = Add, edit, or remove users

# Help section
help-title = Need help setting up your mail server?
help-description = This admin tool is based on flurdy's comprehensive guide for setting up a complete mail server with Postfix, Dovecot, and more.
help-read-guide = Read the complete setup guide →

# Domains
domains-title = Domains
domains-description = Manage your mail server domains and their settings.
domains-add = Add Domain
domains-table-header-domain = Domain
domains-table-header-status = Status
domains-table-header-actions = Actions
domains-empty-title = No domains
domains-empty-description = Get started by creating a new domain.
domains-add-title = Add Domain
domains-edit-title = Edit Domain
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
domains-enable-disable-domain = Enable/Disable Domain
domains-enable-domain = Enable Domain
domains-disable-domain = Disable Domain
domains-delete-domain = Delete Domain
domains-delete-confirm = Are you sure you want to delete this domain?
domains-not-found = Domain not found

# Backups
backups-title = Backups
backups-description = Manage your backup mail servers and their settings.
backups-add = Add Backup
backups-table-header-domain = Domain
backups-table-header-transport = Transport
backups-table-header-status = Status
backups-table-header-actions = Actions
backups-empty-title = No backups
backups-empty-description = Get started by creating a new backup server.
backups-table-header-created = Created
backups-view = View
backups-enable = Enable
backups-disable = Disable
backups-empty-no-backup-servers = No backup servers
backups-empty-get-started = Get started by creating a new backup server.
backups-show-title = Backup
backups-view-edit-settings = View and edit backup server settings.
backups-back-to-backups = Back to Backups
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
backups-add-title = Add Backup
backups-edit-title = Edit Backup
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
aliases-table-header-source = Source
aliases-table-header-destination = Destination
aliases-table-header-domain = Domain
aliases-table-header-status = Status
aliases-table-header-actions = Actions
aliases-empty-title = No aliases
aliases-empty-description = Get started by creating a new alias.
aliases-add-title = Add Alias
aliases-edit-title = Edit Alias
aliases-show-title = Alias
aliases-table-header-mail = Mail
aliases-enable-alias = Enable
aliases-disable-alias = Disable
aliases-view-edit-settings = View and edit alias settings.
aliases-back-to-aliases = Back to Aliases
aliases-alias-information = Alias Information
aliases-alias-details = Alias details and configuration.
aliases-mail = Mail
aliases-forward-to = Forward To
aliases-domain = Domain
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
aliases-domain = Domain
aliases-placeholder-mail = alias@example.com
aliases-placeholder-destination = destination@example.com
aliases-placeholder-domain = example.com
aliases-tooltip-mail = The email address for the alias (e.g., alias@example.com)
aliases-tooltip-destination = The destination email address where mail will be forwarded
aliases-tooltip-domain = The domain for this alias (must match an existing domain)
aliases-active = Active
aliases-tooltip-active = Enable this alias for mail forwarding
aliases-cancel = Cancel
aliases-update-alias = Update Alias
aliases-create-alias = Create Alias

# Users
users-title = Users
users-description = Manage your mail server users and their accounts.
users-add = Add User
users-table-header-username = Username
users-table-header-domain = Domain
users-table-header-quota = Quota
users-table-header-status = Status
users-table-header-actions = Actions
users-empty-title = No users
users-empty-description = Get started by creating a new user.
users-add-title = Add User
users-edit-title = Edit User
users-show-title = User
users-table-header-user-id = User ID
users-table-header-name = Name
users-view = View
users-enable = Enable
users-disable = Disable
users-empty-no-users = No users
users-empty-get-started = Get started by creating a new user.
users-show-user-title = User
users-view-edit-settings = View and edit user settings.
users-back-to-users = Back to Users
users-user-information = User Information
users-user-details = User details and configuration.
users-user-id = User ID
users-full-name = Full Name
users-domain = Domain
users-status = Status
users-created = Created
users-modified = Modified
users-edit-user = Edit User
users-enable-user = Enable User
users-disable-user = Disable User
users-delete-user = Delete User
users-delete-confirm = Are you sure you want to delete this user?
users-new-user = New User
users-edit-user-title = Edit User
users-form-user-id = User ID
users-form-password = Password
users-form-name = Name
users-form-domain = Domain
users-form-active = Active
users-placeholder-user-email = user@example.com
users-placeholder-name = John Doe
users-placeholder-domain = example.com
users-tooltip-user-id = The email address for the user account (e.g., user@example.com)
users-tooltip-password = The password for the user account (leave empty to keep existing password when editing)
users-tooltip-name = The display name for the user (e.g., John Doe)
users-tooltip-domain = The domain for this user (must match an existing domain)
users-tooltip-active = Enable this user account
users-cancel = Cancel
users-create-user = Create User
users-update-user = Update User

# Statistics
stats-title = Statistics
stats-description = View detailed statistics about your mail server.
stats-system-overview = System Overview
stats-system-description = System-wide and per-domain statistics for your mail server.
stats-total-domains = Total Domains
stats-total-backups = Total Backups
stats-total-aliases = Total Aliases
stats-total-users = Total Users
stats-domain-statistics = Domain Statistics
stats-table-header-domain = Domain
stats-table-header-users = Users
stats-table-header-aliases = Aliases
stats-table-header-total-quota = Total Quota
stats-table-header-used-quota = Used Quota

# Forms
form-domain = Domain
form-transport = Transport
form-enabled = Enabled
form-username = Username
form-password = Password
form-quota = Quota
form-source = Source
form-destination = Destination

# Form validation
validation-domain-required = Domain name is required. Please enter a valid domain name.
validation-username-required = Username is required.
validation-password-required = Password is required.
validation-quota-required = Quota is required.
validation-source-required = Source email is required.
validation-destination-required = Destination email is required.

# Error messages
error-unexpected = An unexpected error occurred. Please try again.
error-not-found = Not found
error-duplicate-domain = A domain with this name already exists.
error-duplicate-backup = A backup server for domain '{ $domain }' already exists.
error-duplicate-alias = An alias with this source already exists.
error-duplicate-user = A user with this username already exists.
error-constraint-violation = The data does not meet the required constraints. Please check your input.

# Success messages
success-created = Successfully created.
success-updated = Successfully updated.
success-deleted = Successfully deleted.
success-enabled = Successfully enabled.
success-disabled = Successfully disabled.

# Theme
theme-toggle = Toggle theme

# Login
login-title = Sign in to Sorting Office
login-user-id = User ID
login-password = Password
login-sign-in = Sign in
login-error = Error

# About
about-title = About Sorting Office
about-description = A comprehensive mail server administration tool for managing domains, users, aliases, and backups.
about-what-is-title = What is Sorting Office?
about-what-is-description-1 = Sorting Office is a web-based administration interface for managing a complete mail server setup. It provides an intuitive way to manage domains, users, email aliases, and backup configurations for a Postfix and Dovecot mail server environment.
about-what-is-description-2 = Built with Rust and modern web technologies, Sorting Office offers a secure, fast, and reliable way to administer your mail server without needing to manually edit configuration files.
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
about-feature-dark-mode = Dark Mode Support
about-feature-dark-mode-desc = Modern UI with dark mode for comfortable use
about-tech-stack-title = Technology Stack
about-tech-backend = Backend
about-tech-backend-desc = Rust with Axum web framework
about-tech-database = Database
about-tech-database-desc = MySQL with Diesel ORM
about-tech-frontend = Frontend
about-tech-frontend-desc = HTML templates with Tailwind CSS
about-tech-templating = Templating
about-tech-templating-desc = Askama template engine
about-tech-mail-server = Mail Server
about-tech-mail-server-desc = Postfix & Dovecot
about-tech-deployment = Deployment
about-tech-deployment-desc = Docker containerization
about-flurdy-guide-title = Based on flurdy's Complete Mail Server Guide
about-flurdy-guide-description = This administration tool is designed to work with the comprehensive mail server setup guide created by flurdy, which covers Postfix, Dovecot, SpamAssassin, and more.
about-flurdy-guide-link = Read the complete setup guide →
about-github-title = GitHub Project
about-github-open-source = Open Source
about-github-open-source-desc = Sorting Office is an open-source project hosted on GitHub under the MIT license.
about-github-view-repo = View Repository
about-github-view-repo-desc = Browse source code and documentation
about-github-report-issues = Report Issues
about-github-report-issues-desc = Bug reports and feature requests
about-github-pull-requests = Pull Requests
about-github-pull-requests-desc = Contribute to the project
about-github-readme = README
about-github-readme-desc = Project documentation and setup
about-version-title = Version Information
about-version-project-details = Project Details
about-version-version = Version
about-version-license = License
about-version-maintainer = Maintainer

# Forms
form-error = Error
form-cancel = Cancel
form-create = Create
form-update = Update
form-edit = Edit
form-new = New
form-active = Active
form-placeholder-domain = example.com
form-placeholder-transport = virtual
form-tooltip-domain = The domain name (e.g., example.com)
form-tooltip-transport = Transport configuration (e.g., virtual for virtual domains)
form-tooltip-enable = Enable this domain
form-create-domain = Create Domain
form-update-domain = Update Domain
