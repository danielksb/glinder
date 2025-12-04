const card = document.getElementById('card');
let actionBar = null; // we'll rebind after rendering the card content
let startX = 0;
let currentX = 0;
let isDragging = false;

async function loadInitialImage() {
    const params = new URLSearchParams(window.location.search);
    const id = params.get('id');
    
    if (id) {
        try {
            const response = await fetch(`/api/meta/${id}`);
            if (!response.ok) throw new Error('Failed to load specific image');
            const data = await response.json();
            updateCard(data, true);
            return;
        } catch (err) {
            console.error("Could not load specific image, falling back to random", err);
        }
    }
    loadNextImage(true);
}

async function loadNextImage(replace = false) {
    try {
        const response = await fetch('/api/next');
        if (!response.ok) throw new Error('Failed to load');
        const data = await response.json();
        
        updateCard(data, replace);
    } catch (err) {
        console.error(err);
        card.innerHTML = '<div class="loading">No more profiles (or error)</div>';
    }
}

function updateCard(data, replace = false) {
    // Update URL
    const newUrl = `${window.location.pathname}?id=${data.id}`;
    if (replace) {
        window.history.replaceState({path: newUrl}, '', newUrl);
    } else {
        window.history.pushState({path: newUrl}, '', newUrl);
    }

    // Reset card position
    card.style.transform = '';
    card.style.opacity = '1';
    if (actionBar) actionBar.style.display = '';
    
    // Build safe DOM nodes instead of injecting raw HTML to avoid XSS
    // Include the action bar markup inside the card so it remains part of the card DOM
    card.innerHTML = `
        <img class="card-image" alt="Profile Picture" draggable="false">
        <div class="card-content">
            <div class="card-title"></div>
            <!-- Descriptions are toggled with the info button; actionBar sits below the title and below the description when shown -->
            <div class="card-text" id="cardText" style="display:none"></div>
            <div id="actionBar" class="controls action-bar" role="toolbar" aria-label="Actions">
                <button class="btn btn-nope" data-action="nope">✖</button>
                <button class="btn btn-info" aria-expanded="false">ℹ️</button>
                <button class="btn btn-like" data-action="like">♥</button>
            </div>
        </div>
    `;

    // Rebind the actionBar after parsing the card content
    actionBar = card.querySelector('#actionBar');

    // Populate fields using textContent to keep user content safe and CSS pre-wrap to preserve newlines
    const imgEl = card.querySelector('.card-image');
    const titleEl = card.querySelector('.card-title');
    const textEl = card.querySelector('.card-text');

    if (imgEl) imgEl.setAttribute('src', data.url);
    if (titleEl) titleEl.textContent = data.name || '';
    if (textEl) textEl.textContent = data.description || '';

    // Default: hide text
    if (textEl) textEl.style.display = 'none';
    // Toggle the description visible/hidden above the single actionBar
    function showDescription() {
        if (textEl) textEl.style.display = '';
        // Ensure enough scrollable area so the bottom of the text isn't overlapped by the action bar
        const c = card.querySelector('.card-content');
        if (c) c.scrollTop = 0; // keep at top - user can scroll to see the bottom
        // Update the accessibility attribute on the info button
        if (actionBar) {
            const actionInfo = actionBar.querySelector('.btn-info');
            if (actionInfo) actionInfo.setAttribute('aria-expanded', 'true');
        }
    }

    function hideDescription() {
        if (textEl) textEl.style.display = 'none';
        if (actionBar) {
            const actionInfo = actionBar.querySelector('.btn-info');
            if (actionInfo) actionInfo.setAttribute('aria-expanded', 'false');
        }
    }

    // Wire actionBar buttons (if present) to same handlers — replace onclick each render
    if (actionBar) {
        const actionInfo = actionBar.querySelector('.btn-info');
        const actionLike = actionBar.querySelector('.btn-like');
        const actionNope = actionBar.querySelector('.btn-nope');
        if (actionInfo) {
            // Use event listeners so we can stop propagation on touch events
            actionInfo.addEventListener('click', (ev) => { ev.stopPropagation(); if (textEl && textEl.style.display === 'none') showDescription(); else hideDescription(); });
            actionInfo.addEventListener('touchstart', (ev) => { ev.stopPropagation(); });
            actionInfo.addEventListener('touchend', (ev) => { ev.stopPropagation(); });
            // Reset aria state for new card and connect to the card text
            actionInfo.setAttribute('aria-expanded', 'false');
            actionInfo.setAttribute('aria-controls', 'cardText');
        }
        if (actionLike) {
            actionLike.addEventListener('click', (ev) => { ev.stopPropagation(); swipe('right'); });
            actionLike.addEventListener('touchstart', (ev) => { ev.stopPropagation(); });
            actionLike.addEventListener('touchend', (ev) => { ev.stopPropagation(); });
        }
        if (actionNope) {
            actionNope.addEventListener('click', (ev) => { ev.stopPropagation(); swipe('left'); });
            actionNope.addEventListener('touchstart', (ev) => { ev.stopPropagation(); });
            actionNope.addEventListener('touchend', (ev) => { ev.stopPropagation(); });
        }
    }
}

window.addEventListener('popstate', () => {
    loadInitialImage();
});

function swipe(direction) {
    const screenWidth = window.innerWidth;
    const endX = direction === 'right' ? screenWidth : -screenWidth;
    
    card.style.transition = 'transform 0.5s ease, opacity 0.5s ease';
    card.style.transform = `translate(${endX}px, 0) rotate(${direction === 'right' ? 20 : -20}deg)`;
    card.style.opacity = '0';
    // Action bar is anchored to the card; it will move with the card transform and does not need separate translation

    setTimeout(() => {
        card.style.transition = 'none';
        card.style.transform = 'translate(0, 0) rotate(0)';
        if (actionBar) {
            actionBar.style.transition = 'none';
            actionBar.style.transform = 'none';
        }
        card.style.opacity = '1';
        // Show loading state briefly or just keep old content until new loads?
        // Let's show loading to be clear
        card.innerHTML = '<div class="loading">Finding match...</div>';
        loadNextImage();
    }, 500);
}

// Touch events for swipe
card.addEventListener('touchstart', (e) => {
    // If the touch started on the controls (buttons), don't begin dragging
    try {
        if (e.target && e.target.closest && e.target.closest('.controls')) {
            isDragging = false;
            return;
        }
    } catch (err) {
        // ignore errors and fall back to default behavior
    }
    startX = e.touches[0].clientX;
    isDragging = true;
    card.style.transition = 'none';
});

card.addEventListener('touchmove', (e) => {
    if (!isDragging) return;
    currentX = e.touches[0].clientX;
    const diffX = currentX - startX;
    const rotation = diffX / 20;
    card.style.transform = `translate(${diffX}px, 0) rotate(${rotation}deg)`;
    // Keep the fixed action bar static while dragging the card; do not translate it
});

card.addEventListener('touchend', (e) => {
    isDragging = false;
    const diffX = currentX - startX;
    const threshold = 100;

    if (Math.abs(diffX) > threshold) {
        swipe(diffX > 0 ? 'right' : 'left');
    } else {
        card.style.transition = 'transform 0.3s ease';
        card.style.transform = 'translate(0, 0) rotate(0)';
    }
});

// Initial load
loadInitialImage();
