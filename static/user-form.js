// User form functionality

document.addEventListener("DOMContentLoaded", function () {
  try {
    const userIdInput = document.getElementById("id");
    const domainSearchResults = document.getElementById("user-domain-search-results");

    if (!userIdInput || !domainSearchResults) {
      console.error("Required elements not found");
      return;
    }

    // Show/hide domain search results based on input focus
    userIdInput.addEventListener("focus", function () {
      if (domainSearchResults.innerHTML.trim() !== "") {
        domainSearchResults.classList.remove("hidden");
      }
    });

    // Hide search results when clicking outside
    document.addEventListener("click", function (e) {
      if (
        !userIdInput.contains(e.target) &&
        !domainSearchResults.contains(e.target)
      ) {
        domainSearchResults.classList.add("hidden");
      }
    });

    // Handle clicking on domain search results
    domainSearchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li) {
        const domain = li.getAttribute("data-domain") || "";
        if (domain) {
          // Get current value and extract the part before @
          const currentValue = userIdInput.value;
          const atIndex = currentValue.indexOf('@');
          let prefix = "";
          if (atIndex !== -1) {
            prefix = currentValue.substring(0, atIndex);
          } else {
            // If no @ found, use the current value as prefix
            prefix = currentValue;
          }

          // Set the new value with the selected domain
          userIdInput.value = prefix + "@" + domain;
          domainSearchResults.classList.add("hidden");
          userIdInput.focus();
        }
      }
    });

    // Handle HTMX after swap to show results if they exist
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "user-domain-search-results") {
        const query = userIdInput.value.trim();
        if (query.length >= 2 && domainSearchResults.innerHTML.trim() !== "") {
          domainSearchResults.classList.remove("hidden");
        } else {
          domainSearchResults.classList.add("hidden");
        }
      }
    });
  } catch (error) {
    console.error("Error in user form JavaScript:", error);
  }
});
