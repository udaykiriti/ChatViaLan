// ===== Main Entry Point =====

function init() {
    initDOM();
    initTheme();
    initSound();
    initMobileMenu();
    buildEmojiPicker();
    initEventListeners();
    updateInputState();
    connect();

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
