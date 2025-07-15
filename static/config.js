// Config page functionality

function addRequiredAlias() {
    const input = document.querySelector('input[name="new_required_alias"]');
    const alias = input.value.trim();
    if (!alias) return;

    const aliasesList = document.getElementById('required-aliases-list');
    const aliasDiv = document.createElement('div');
    aliasDiv.className = 'flex items-center justify-between p-3 bg-red-50 dark:bg-red-900/20 rounded-md';
    aliasDiv.innerHTML = `
        <span class="text-sm font-medium text-red-900 dark:text-red-100">${alias}</span>
        <div class="flex items-center space-x-2">
            <button type="button" onclick="demoteToCommon('${alias}')" class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 text-xs">
                Demote to Common
            </button>
            <button type="button" onclick="removeRequiredAlias('${alias}')" class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300">
                Remove
            </button>
        </div>
        <input type="hidden" name="required_aliases[]" value="${alias}">
    `;
    aliasesList.appendChild(aliasDiv);
    input.value = '';
}

function removeRequiredAlias(alias) {
    const aliasesList = document.getElementById('required-aliases-list');
    const aliasDivs = aliasesList.querySelectorAll('div');
    aliasDivs.forEach(div => {
        const span = div.querySelector('span');
        if (span && span.textContent === alias) {
            div.remove();
        }
    });
}

function demoteToCommon(alias) {
    // Implementation of demoteToCommon function
    console.log('Demote to common:', alias);
}

function addCommonAlias() {
    const input = document.querySelector('input[name="new_common_alias"]');
    const alias = input.value.trim();
    if (!alias) return;

    const aliasesList = document.getElementById('common-aliases-list');
    const aliasDiv = document.createElement('div');
    aliasDiv.className = 'flex items-center justify-between p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-md';
    aliasDiv.innerHTML = `
        <span class="text-sm font-medium text-yellow-900 dark:text-yellow-100">${alias}</span>
        <div class="flex items-center space-x-2">
            <button type="button" onclick="promoteToRequired('${alias}')" class="text-green-600 hover:text-green-800 dark:text-green-400 dark:hover:text-green-300 text-xs">
                Promote to Required
            </button>
            <button type="button" onclick="removeCommonAlias('${alias}')" class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300">
                Remove
            </button>
        </div>
        <input type="hidden" name="common_aliases[]" value="${alias}">
    `;
    aliasesList.appendChild(aliasDiv);
    input.value = '';
}

function removeCommonAlias(alias) {
    const aliasesList = document.getElementById('common-aliases-list');
    const aliasDivs = aliasesList.querySelectorAll('div');
    aliasDivs.forEach(div => {
        const span = div.querySelector('span');
        if (span && span.textContent === alias) {
            div.remove();
        }
    });
}

function promoteToRequired(alias) {
    // Implementation of promoteToRequired function
    console.log('Promote to required:', alias);
}

function addDomainOverride() {
    const input = document.querySelector('input[name="new_domain"]');
    const domain = input.value.trim();
    if (!domain) return;

    const overridesList = document.getElementById('domain-overrides-list');
    const domainDiv = document.createElement('div');
    domainDiv.className = 'border border-gray-200 dark:border-gray-600 rounded-md p-4';
    domainDiv.innerHTML = `
        <div class="flex items-center justify-between mb-3">
            <h4 class="text-sm font-medium text-gray-900 dark:text-white">${domain}</h4>
            <button type="button" onclick="removeDomainOverride('${domain}')" class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300">
                Remove Domain
            </button>
        </div>
        <div class="space-y-2">
            <div class="flex items-center space-x-4">
                <input type="text" placeholder="Domain alias" class="flex-1 rounded-md border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm">
                <button type="button" onclick="addDomainAlias('${domain}')" class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                    Add Alias
                </button>
            </div>
        </div>
        <input type="hidden" name="domain_overrides[${domain}][]" value="">
    `;
    overridesList.appendChild(domainDiv);
    input.value = '';
}

function removeDomainOverride(domain) {
    const overridesList = document.getElementById('domain-overrides-list');
    const domainDivs = overridesList.querySelectorAll('div');
    domainDivs.forEach(div => {
        const h4 = div.querySelector('h4');
        if (h4 && h4.textContent === domain) {
            div.remove();
        }
    });
}

function addDomainAlias(domain) {
    const domainDiv = event.target.closest('div');
    const input = domainDiv.querySelector('input[type="text"]');
    const alias = input.value.trim();
    if (!alias) return;

    const aliasesContainer = domainDiv.querySelector('.space-y-2');
    const aliasDiv = document.createElement('div');
    aliasDiv.className = 'flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded';
    aliasDiv.innerHTML = `
        <span class="text-sm text-gray-900 dark:text-white">${alias}</span>
        <button type="button" onclick="removeDomainAlias('${domain}', '${alias}')" class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300">
            Remove
        </button>
        <input type="hidden" name="domain_overrides[${domain}][]" value="${alias}">
    `;
    aliasesContainer.appendChild(aliasDiv);
    input.value = '';
}

function removeDomainAlias(domain, alias) {
    const domainDiv = event.target.closest('div');
    const aliasDivs = domainDiv.querySelectorAll('div');
    aliasDivs.forEach(div => {
        const span = div.querySelector('span');
        if (span && span.textContent === alias) {
            div.remove();
        }
    });
}

function removeDomainRequiredAlias(domain, alias) {
    // Implementation of removeDomainRequiredAlias function
    console.log('Remove domain required alias:', domain, alias);
}

function removeDomainCommonAlias(domain, alias) {
    // Implementation of removeDomainCommonAlias function
    console.log('Remove domain common alias:', domain, alias);
}
