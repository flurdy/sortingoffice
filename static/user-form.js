console.log("[domain-suggest] user-form.js loaded");
// User form functionality

try {
  const userIdInput = document.getElementById("id");
  const searchResults = document.getElementById("user-id-search-results");
  const userIdDestinationInput = document.getElementById("user-id-destination");

  if (!userIdInput) {
    console.warn("[user-suggest] userIdInput not found");
  }
  if (!searchResults) {
    console.warn("[user-suggest] searchResults not found");
  }
  if (!userIdDestinationInput) {
    console.warn("[user-suggest] userIdDestinationInput not found");
  }
  if (userIdInput && searchResults && userIdDestinationInput) {
    userIdInput.addEventListener("input", function () {
      const value = userIdInput.value;
      // Only trigger if input is at least 2 chars
      if (value.trim().length >= 2) {
        userIdDestinationInput.value = value;
        userIdDestinationInput.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
        console.log("[user-suggest] Dispatching input event on user-id-destination", value);
      } else {
        searchResults.classList.add("hidden");
        console.log("[user-suggest] Hiding user id suggestions (input too short)");
      }
    });
    // Show/hide search results after HTMX swap
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "user-id-search-results") {
        const query = userIdInput.value.trim();
        if (query.length >= 2 && searchResults.innerHTML.trim() !== "") {
          searchResults.classList.remove("hidden");
          console.log("[user-suggest] Showing user id suggestions");
        } else {
          searchResults.classList.add("hidden");
          console.log("[user-suggest] Hiding user id suggestions (no results)");
        }
      }
    });
    // Insert clicked suggestion into input
    searchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li && userIdInput) {
        const value = li.textContent.trim();
        if (value) {
          userIdInput.value = value;
          searchResults.classList.add("hidden");
          userIdInput.focus();
          console.log("[user-suggest] Inserted '", value, "' into user id input");
        }
      }
    });
  }
} catch (err) {
  console.error("[user-suggest] Error in user-form.js:", err);
}
