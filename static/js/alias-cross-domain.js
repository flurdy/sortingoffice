// Alias cross-domain search functionality

document.addEventListener("DOMContentLoaded", function () {
  try {
    const aliasInput = document.getElementById("alias");
    const searchResults = document.getElementById("alias-search-results");

    if (!aliasInput || !searchResults) {
      console.error("Required elements not found");
      return;
    }

    // Show/hide search results based on input focus
    aliasInput.addEventListener("focus", function () {
      if (searchResults.innerHTML.trim() !== "") {
        searchResults.classList.remove("hidden");
      }
    });

    // Hide search results when clicking outside
    document.addEventListener("click", function (e) {
      if (
        !aliasInput.contains(e.target) &&
        !searchResults.contains(e.target)
      ) {
        searchResults.classList.add("hidden");
      }
    });

    // Handle clicking on search results
    searchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li) {
        // Extract just the alias part (before @) from the mail address
        const mailAddress = li.getAttribute("data-alias") || "";
        const aliasPart = mailAddress.split('@')[0];
        if (aliasPart) {
          aliasInput.value = aliasPart;
          searchResults.classList.add("hidden");
          aliasInput.focus();
        }
      }
    });

    // Handle HTMX after swap to show results if they exist
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "alias-search-results") {
        const query = aliasInput.value.trim();
        if (query.length >= 2 && searchResults.innerHTML.trim() !== "") {
          searchResults.classList.remove("hidden");
        } else {
          searchResults.classList.add("hidden");
        }
      }
    });
  } catch (error) {
    console.error("Error in alias cross-domain search JavaScript:", error);
  }
});
