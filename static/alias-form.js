// Alias form functionality

document.addEventListener("DOMContentLoaded", function () {
  try {
    const destinationInput = document.getElementById("destination");
    const searchResults = document.getElementById("search-results");

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

    // Hide search results when clicking outside
    document.addEventListener("click", function (e) {
      if (
        !destinationInput.contains(e.target) &&
        !searchResults.contains(e.target)
      ) {
        searchResults.classList.add("hidden");
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
    });
  } catch (error) {
    console.error("Error in alias form JavaScript:", error);
  }
});
