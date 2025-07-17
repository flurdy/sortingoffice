// Minimal test: log when mail input is found and when its input event fires
var mailInput = document.getElementById("mail");
if (mailInput) {
  console.log("[test] mail input found at script load");
  mailInput.addEventListener("input", function () {
    console.log("[test] mail input event fired, value:", mailInput.value);
  });
} else {
  console.log("[test] mail input not found at script load");
}

console.log("[domain-suggest] alias-form.js loaded");
// Alias form functionality
  try {
    const destinationInput = document.getElementById("destination");
    const searchResults = document.getElementById("search-results");
  const domainSearchResults = document.getElementById("domain-search-results");
  const mailDomainInput = document.getElementById("mail-domain");

  // --- Destination suggestion feature ---
  if (!destinationInput) {
    console.warn("[destination-suggest] destinationInput not found");
  }
  if (!searchResults) {
    console.warn("[destination-suggest] searchResults not found");
  }
  if (destinationInput && searchResults) {
    destinationInput.addEventListener("input", function () {
      console.log("[destination-suggest] input event fired, value:", destinationInput.value);
    });
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "search-results") {
        console.log("[destination-suggest] htmx:afterSwap for search-results");
        console.log("[destination-suggest] searchResults:", searchResults);
        const query = destinationInput.value.trim();
        if (query.length >= 2 && searchResults.innerHTML.trim() !== "") {
          searchResults.classList.remove("hidden");
        } else {
          searchResults.classList.add("hidden");
        }
      }
    });
    // --- Click handler for destination suggestions ---
    searchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li && destinationInput) {
        let valueToInsert = li.getAttribute("data-mail") || li.getAttribute("data-destination") || li.textContent.trim();
        if (valueToInsert) {
          destinationInput.value = valueToInsert;
          searchResults.classList.add("hidden");
          destinationInput.focus();
          console.log("[destination-suggest] Inserted '", valueToInsert, "' into destination input");
        }
      }
    });
  }

  // --- Domain suggestion trigger for mail field ---
  if (!mailInput) {
    console.warn("[domain-suggest] mailInput not found");
  }
  if (!domainSearchResults) {
    console.warn("[domain-suggest] domainSearchResults not found");
  }
  if (!mailDomainInput) {
    console.warn("[domain-suggest] mailDomainInput not found");
  }
  if (mailInput && domainSearchResults && mailDomainInput) {
    mailInput.addEventListener("input", function (e) {
      const value = mailInput.value;
      const atIndex = value.indexOf("@");
      // Only trigger if '@' is present and cursor is after it, and at least one char after '@'
      if (
        atIndex !== -1 &&
        mailInput.selectionStart > atIndex + 1 &&
        value.length > atIndex + 1
      ) {
        // Set the hidden domain input value to the part after '@'
        mailDomainInput.value = value.substring(atIndex + 1);
        console.log("[domain-suggest] Dispatching input event on mail-domain", mailDomainInput.value);
        mailDomainInput.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
      } else {
        // Hide suggestions if not in domain part
            domainSearchResults.classList.add("hidden");
        console.log("[domain-suggest] Hiding domain suggestions (not in domain part)");
        }
      });
    document.body.addEventListener("htmx:afterSwap", function (e) {
      if (e.target.id === "domain-search-results") {
        console.log("[domain-suggest] htmx:afterSwap for domain-search-results");
        const query = mailInput ? mailInput.value.trim() : "";
        if (
          query.length >= 2 &&
          domainSearchResults &&
          domainSearchResults.innerHTML.trim() !== ""
        ) {
          domainSearchResults.classList.remove("hidden");
          console.log("[domain-suggest] Showing domain suggestions");
        } else if (domainSearchResults) {
          domainSearchResults.classList.add("hidden");
          console.log("[domain-suggest] Hiding domain suggestions (no results)");
        }
      }
    });
    // --- Click handler for domain suggestions ---
    domainSearchResults.addEventListener("click", function (e) {
      const li = e.target.closest("li");
      if (li && mailInput) {
        const domain = li.getAttribute("data-domain") || li.textContent.trim();
        if (domain) {
          const currentValue = mailInput.value;
          const atIndex = currentValue.indexOf("@");
          let prefix = "";
          if (atIndex !== -1) {
            prefix = currentValue.substring(0, atIndex);
          } else {
            prefix = currentValue;
          }
          mailInput.value = prefix + "@" + domain;
          domainSearchResults.classList.add("hidden");
          mailInput.focus();
          console.log("[domain-suggest] Inserted domain '", domain, "' into mail input");
        }
  }
});
  }
} catch (err) {
  console.error("[domain-suggest] Error in alias-form.js:", err);
}
