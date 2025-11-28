const card = document.getElementById('card');
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
    
    // Build safe DOM nodes instead of injecting raw HTML to avoid XSS
    card.innerHTML = `
        <img class="card-image" alt="Profile Picture" draggable="false">
        <div class="card-content">
            <div class="card-title"></div>
            <div class="card-text"></div>
        </div>
    `;

    // Populate fields using textContent to keep user content safe and CSS pre-wrap to preserve newlines
    const imgEl = card.querySelector('.card-image');
    const titleEl = card.querySelector('.card-title');
    const textEl = card.querySelector('.card-text');

    if (imgEl) imgEl.setAttribute('src', data.url);
    if (titleEl) titleEl.textContent = data.name || '';
    if (textEl) textEl.textContent = data.description || '';
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

    setTimeout(() => {
        card.style.transition = 'none';
        card.style.transform = 'translate(0, 0) rotate(0)';
        card.style.opacity = '1';
        // Show loading state briefly or just keep old content until new loads?
        // Let's show loading to be clear
        card.innerHTML = '<div class="loading">Finding match...</div>';
        loadNextImage();
    }, 500);
}

// Touch events for swipe
card.addEventListener('touchstart', (e) => {
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
