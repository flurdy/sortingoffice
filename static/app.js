// Theme management
function getTheme() {
    return localStorage.getItem('theme') || 'light';
}

function setTheme(theme) {
    localStorage.setItem('theme', theme);
    document.documentElement.classList.toggle('dark', theme === 'dark');
}

// Initialize theme on page load
document.addEventListener('DOMContentLoaded', function() {
    const theme = getTheme();
    setTheme(theme);
});

// Theme toggle function
function toggleTheme() {
    const currentTheme = getTheme();
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';

    // Send request to server
    fetch('/theme/toggle', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: `theme=${currentTheme}`
    })
    .then(response => response.text())
    .then(newTheme => {
        setTheme(newTheme);
    })
    .catch(error => {
        console.error('Error toggling theme:', error);
        // Fallback to client-side only
        setTheme(newTheme);
    });
}

// Language dropdown toggle function
function toggleLanguageDropdown() {
    const dropdown = document.getElementById('language-dropdown');
    if (dropdown) {
        dropdown.classList.toggle('hidden');
    }
}

// Close language dropdown when clicking outside
document.addEventListener('click', function(event) {
    const dropdown = document.getElementById('language-dropdown');
    const button = event.target.closest('button[onclick="toggleLanguageDropdown()"]');

    if (dropdown && !button && !dropdown.contains(event.target)) {
        dropdown.classList.add('hidden');
    }
});

// Sidebar toggle function
function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        const isOpen = sidebar.getAttribute('data-sidebar-open') === 'true';
        sidebar.setAttribute('data-sidebar-open', !isOpen);
    }
}

// Handle window resize to ensure proper sidebar state
window.addEventListener('resize', function() {
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        const isLargeScreen = window.innerWidth >= 1024;

        if (isLargeScreen) {
            // On large screens, ensure sidebar is visible
            sidebar.setAttribute('data-sidebar-open', 'true');
        } else {
            // On small screens, keep current state or default to closed
            if (sidebar.getAttribute('data-sidebar-open') === null) {
                sidebar.setAttribute('data-sidebar-open', 'false');
            }
        }
    }
});

// Initialize sidebar state on page load
document.addEventListener('DOMContentLoaded', function() {
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        const isLargeScreen = window.innerWidth >= 1024;

        if (isLargeScreen) {
            sidebar.setAttribute('data-sidebar-open', 'true');
        } else {
            sidebar.setAttribute('data-sidebar-open', 'false');
        }
    }
});

// Progress bar initialization
document.addEventListener('DOMContentLoaded', function() {
    const progressBar = document.querySelector('[data-width]');
    if (progressBar) {
        const width = progressBar.getAttribute('data-width');
        progressBar.style.width = width + '%';
    }
});
