// Alias form functionality

document.addEventListener("DOMContentLoaded", function () {
  try {
    const destinationInput = document.getElementById("destination");
    const searchResults = document.getElementById("search-results");
    const mailInput = document.getElementById("mail");
    const domainSearchResults = document.getElementById(
      "domain-search-results"
    );

    if (!destinationInput || !searchResults) {
      console.error("Required elements not found");
      return;
    }

    // Show/hide search results based on input focus
    destinationInput.addEventListener("focus", function () {
      if (searchResults.innerHTML.trim() !== "") {
        searchResults.classList.remove("hidden");
      }
    });

    // Show/hide domain search results based on input focus
    if (mailInput && domainSearchResults) {
      mailInput.addEventListener("focus", function () {
        if (domainSearchResults.innerHTML.trim() !== "") {
          domainSearchResults.classList.remove("hidden");
        }
      });
    }

    // Hide search results when clicking outside
    document.addEventListener("click", function (e) {
      if (
        !destinationInput.contains(e.target) &&
        !searchResults.contains(e.target)
      ) {
        searchResults.classList.add("hidden");
      }

      if (
        mailInput &&
        domainSearchResults &&
        !mailInput.contains(e.target) &&
        !domainSearchResults.contains(e.target)
      ) {
        domainSearchResults.classList.add("hidden");
      }
    });

    // Handle clicking on search results
    searchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li) {
        // If user clicks on a span, use its text; otherwise, use destination
        let valueToInsert = "";
        if (e.target.tagName === "SPAN") {
          valueToInsert = e.target.textContent.trim();
        } else {
          valueToInsert = li.getAttribute("data-destination") || "";
        }
        if (valueToInsert) {
          destinationInput.value = valueToInsert;
          searchResults.classList.add("hidden");
          destinationInput.focus();
        }
      }
    });

    // Handle clicking on domain search results
    if (domainSearchResults) {
      domainSearchResults.addEventListener("click", function (e) {
        const li = e.target.closest("li");
        if (li && mailInput) {
          const domain = li.getAttribute("data-domain") || "";
          if (domain) {
            // Get current value and extract the part before @
            const currentValue = mailInput.value;
            const atIndex = currentValue.indexOf("@");
            let prefix = "";
            if (atIndex !== -1) {
              prefix = currentValue.substring(0, atIndex);
            } else {
              // If no @ found, use the current value as prefix
              prefix = currentValue;
            }

            // Set the new value with the selected domain
            mailInput.value = prefix + "@" + domain;
            domainSearchResults.classList.add("hidden");
            mailInput.focus();
          }
        }
      });
    }

    // Handle HTMX after swap to show results if they exist
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "search-results") {
        const query = destinationInput.value.trim();
        if (query.length >= 2 && searchResults.innerHTML.trim() !== "") {
          searchResults.classList.remove("hidden");
        } else {
          searchResults.classList.add("hidden");
        }
      }

      if (e.target.id === "domain-search-results") {
        const query = mailInput ? mailInput.value.trim() : "";
        if (
          query.length >= 2 &&
          domainSearchResults &&
          domainSearchResults.innerHTML.trim() !== ""
        ) {
          domainSearchResults.classList.remove("hidden");
        } else if (domainSearchResults) {
          domainSearchResults.classList.add("hidden");
        }
      }
    });
  } catch (error) {
    console.error("Error in alias form JavaScript:", error);
  }
});
